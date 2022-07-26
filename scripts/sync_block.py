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
        print("reponse is not ok, res is", j_res)
        return 0
    conn.close()
    return j_res["result"]["head"]["number"]


def check_or_do(network):

    # get last_export_height
    last_export_height = ""
    down_last_export_height_file_cmd = "wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/%s/last_export_height.txt -O ./last_export_height.txt" % network
    os.system(down_last_export_height_file_cmd)
    with open('./last_export_height.txt', 'r') as f:
        last_export_height = int(f.readline())

    url = 'https://%s-seed.starcoin.org/' % network
    method = 'POST'
    post_data = '{"jsonrpc":"2.0","method":"chain.info","params":[],"id":0}'
    headers = {"content-type": "application/json"}

    current_height = get_height(method, url, post_data, headers)
    print("main current_height is %s, last_export_height is %s" %
          (current_height, last_export_height))
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
        cp_file_tar_cmd = "timeout 30 bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-%s/starcoin-1/starcoin/sc-data/%s --dst s3://main.starcoin.org/%s/%s '" % (
            network, file_compress_name, network, file_compress_name)
        os.system(cp_file_tar_cmd)

        # cp block list tar to s3
        cp_blocklist_tar_cmd = "timeout 30 bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-%s/starcoin-1/starcoin/sc-data/%s --dst s3://main.starcoin.org/%s/%s '" % (
            network, block_list_file_tar_name, network, block_list_file_tar_name)
        os.system(cp_blocklist_tar_cmd)

        # update the last_export_height
        os.system("echo %s > ./last_export_height.txt" % end)
        os.system("aws s3api put-object --bucket main.starcoin.org --key %s/last_export_height.txt --body ./last_export_height.txt" % network)

        # back up last snapshot cp to backdir
        # do the increment snapshot export
        # check the snapshot is ok or not with the manifest file
        # if not, recover the backdir, then exit.
        # if ok, rm backup dir, and do the next step
        os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'cp -r /sc-data/snapshot /sc-data/snapshotbak'" % network)
        # export snapshot
        export_snapshot_cmd = "kubectl exec -it -n starcoin-%s starcoin-1 -- /starcoin/starcoin_db_exporter export-snapshot --db-path /sc-data/%s -n %s -o /sc-data/snapshot -t true" % (
            network, network, network)
        os.system(export_snapshot_cmd)
        export_state_node_status = os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(less /sc-data/snapshot/manifest.csv| grep state_node | awk -F ' ' '{print\$2}') -eq \$(less /sc-data/snapshot/state_node | wc -l) ]; then exit 0; else exit 1;fi\"" % network)
        export_acc_node_transaction_status = os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(less /sc-data/snapshot/manifest.csv| grep acc_node_transaction | awk -F ' ' '{print\$2}') -eq \$(less /sc-data/snapshot/acc_node_transaction | wc -l) ]; then exit 0; else exit 1;fi\"" % network)
        export_acc_node_block_status = os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(less /sc-data/snapshot/manifest.csv| grep acc_node_block | awk -F ' ' '{print\$2}') -eq \$(less /sc-data/snapshot/acc_node_block | wc -l) ]; then exit 0; else exit 1;fi\"" % network)
        export_block_status = os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(less /sc-data/snapshot/manifest.csv| grep -w block | awk -F ' ' '{print\$2}') -eq \$(less /sc-data/snapshot/block | wc -l) ]; then exit 0; else exit 1;fi\"" % network)
        export_block_info_status = os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(less /sc-data/snapshot/manifest.csv| grep block_info | awk -F ' ' '{print\$2}') -eq \$(less /sc-data/snapshot/block_info | wc -l) ]; then exit 0; else exit 1;fi\"" % network)

        if export_state_node_status != 0 or export_acc_node_transaction_status != 0 or export_acc_node_block_status != 0 or export_block_status != 0 or export_block_info_status != 0:
            os.system("kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'rm -rf /sc-data/snapshot'" % network)
            os.system("kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'mv /sc-data/snapshotbak /sc-data/snapshot'" % network)
            sys.exit(1)
        os.system("kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'rm -rf /sc-data/snapshotbak'" % network)

        # tar snapshot
        tar_snapshot_cmd = "kubectl exec -it -n starcoin-%s starcoin-1 -- tar -czvf /sc-data/%s -C /sc-data/ %s " % (
            network, "snapshot.tar.gz", "snapshot")
        os.system(tar_snapshot_cmd)

        # cp snapshot.tar.gz to s3
        cp_snapshot_tar_cmd = "timeout 30 bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-%s/starcoin-1/starcoin/sc-data/%s --dst s3://main.starcoin.org/%s/%s '" % (
            network, "snapshot.tar.gz", network, "snapshot.tar.gz")
        os.system(cp_snapshot_tar_cmd)


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
