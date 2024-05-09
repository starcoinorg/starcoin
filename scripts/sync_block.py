import os
import sys
import json
import time
import http.client
import urllib.parse
import argparse


def get_height(method, url, post_data, headers):

    parsed_url = urllib.parse.urlparse(url)
    print("begin get_height, %s" % parsed_url.hostname)
    conn = http.client.HTTPSConnection(parsed_url.hostname, parsed_url.port)
    conn.request(method, url, str(post_data), headers)
    response = conn.getresponse()

    string_res = response.read().decode("utf-8")
    j_res = json.loads(string_res)

    print("get_height response status is %s" % response.status)
    if response.status != 200:
        print("response is not ok, res is", j_res)
        return 0
    conn.close()
    return j_res["result"]["head"]["number"]


def check_or_do(network):

    # get last_export_height
    last_export_height = ""
    down_last_export_height_file_cmd = "wget https://s3.ap-northeast-1.amazonaws.com/main1.starcoin.org/%s/last_export_height.txt -O ./last_export_height.txt" % network
    os.system(down_last_export_height_file_cmd)
    with open('./last_export_height.txt', 'r') as f:
        last_export_height = int(f.readline())

    url = 'https://%s-seed.starcoin.org/' % network
    method = 'POST'
    post_data = '{"jsonrpc":"2.0","method":"chain.info","params":[],"id":0}'
    headers = {"content-type": "application/json"}

    current_height = get_height(method, url, post_data, headers)
    print("%s current_height is %s, last_export_height is %s" %
          (network, current_height, last_export_height))
    export_height = int(current_height) - 1000

    if export_height - int(last_export_height) > 10000:

        # export block, kubectl exec
        export_tmp = "kubectl exec -it -n starcoin-%s starcoin-1 -- /starcoin/starcoin_db_exporter export-block-range --db-path /sc-data/%s -s %s -e %s -n %s -o /sc-data/."
        start = last_export_height + 1
        end = last_export_height + 10000
        export_cmd = export_tmp % (network, network, start, end, network)
        print(export_cmd)
        os.system(export_cmd)

        # tar block csv file
        filename = "block_%s_%s.csv" % (start, end)
        file_compress_name = "%s.tar.gz" % filename
        tar_cmd = "kubectl exec -it -n starcoin-%s starcoin-1 -- tar -czvf /sc-data/%s -C /sc-data/ %s " % (
            network, file_compress_name, filename)
        print(tar_cmd)
        os.system(tar_cmd)

        # append and tar the block list csv file
        block_list_file_name = "block_list.csv"
        append_cmd = "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'echo %s >> /sc-data/%s' " % (
            network, filename, block_list_file_name)
        block_list_file_tar_name = "%s.tar.gz" % block_list_file_name
        tar_block_list_cmd = "kubectl exec -it -n starcoin-%s starcoin-1 -- tar -czvf /sc-data/%s -C /sc-data/ %s " % (
            network, block_list_file_tar_name, block_list_file_name)
        os.system(append_cmd)
        os.system(tar_block_list_cmd)

        # cp block file tar to s3
        cp_file_tar_cmd = "timeout 30 bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-%s/starcoin-1/starcoin/sc-data/%s --dst s3://main1.starcoin.org/%s/%s '" % (
            network, file_compress_name, network, file_compress_name)
        os.system(cp_file_tar_cmd)

        # cp block list tar to s3
        cp_blocklist_tar_cmd = "timeout 30 bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-%s/starcoin-1/starcoin/sc-data/%s --dst s3://main1.starcoin.org/%s/%s '" % (
            network, block_list_file_tar_name, network, block_list_file_tar_name)
        os.system(cp_blocklist_tar_cmd)

        # update the last_export_height
        os.system("echo %s > ./last_export_height.txt" % end)
        os.system("aws s3api put-object --bucket main1.starcoin.org --key %s/last_export_height.txt --body ./last_export_height.txt" % network)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description='sync blocks for main, proxima, barnard')
    parser.add_argument(
        '--net',
        metavar='net',
        type=str,
        required=True,
        help='the network to sync'
    )
    args = parser.parse_args()
    network = args.net
    if network not in ["main", "proxima", "barnard"]:
        print("network param is not right %s" % network)
        sys.exit(1)
    check_or_do(network)
