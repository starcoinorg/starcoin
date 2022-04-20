import os
import json
import time
import http.client
import urllib.parse


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


def check_or_do():

    # get last_export_height
    last_export_height = ""
    down_last_export_height_file_cmd = "wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/main/last_export_height.txt -O ./last_export_height.txt"
    os.system(down_last_export_height_file_cmd)
    with open('./last_export_height.txt', 'r') as f:
        last_export_height = f.readline()

    # todo support get network from action env
    url = 'https://main-seed.starcoin.org/'
    method = 'POST'
    post_data = '{"jsonrpc":"2.0","method":"chain.info","params":[],"id":0}'
    headers = {"content-type": "application/json"}

    current_height = get_height(method, url, post_data, headers)
    print("main current_height is %s, last_export_height is %s" %
          (current_height, last_export_height))

    if int(current_height) - int(last_export_height) > 100000:

        # export block, kubectl exec
        export_tmp = "kubectl exec -it -n starcoin-main starcoin-1 -- /starcoin/starcoin_db_exporter export-block-range --db-path /sc-data/main -s %s -e %s -n main -o /sc-data/."
        start = last_export_height + 1
        end = last_export_height + 100000
        export_cmd = export_tmp % (start, end)
        print(export_cmd)
        os.system(export_cmd)
        # wait 180s for 100k main block first time export to finish
        time.sleep(180)

        # tar block csv file
        filename = "block_%s_%s.csv" % (start, end)
        file_compress_name = "%s.tar.gz" % filename
        tar_cmd = "kubectl exec -it -n starcoin-main starcoin-1 -- tar -czvf /sc-data/%s -C /sc-data/ %s " % (
            file_compress_name, filename)
        print(tar_cmd)
        os.system(tar_cmd)
        time.sleep(10)

        # append and tar the block list csv file
        # block_list_file_name = "block_list-test.csv"
        # append_cmd = "kubectl exec -it -n starcoin-main starcoin-1 -- bash -c 'echo %s >> /sc-data/%s' " % (filename, block_list_file_name)
        # block_list_file_tar_name = "%s.tar.gz" % block_list_file_name
        # tar_block_list_cmd = "kubectl exec -it -n starcoin-main starcoin-1 -- tar -czvf /sc-data/%s -C /sc-data/ %s " % (
        #     block_list_file_tar_name, block_list_file_name)
        # os.system(append_cmd)
        # time.sleep(10)
        # os.system(tar_block_list_cmd)
        # time.sleep(10)

        # cp block file tar to s3
        cp_file_tar_cmd = "timeout 30 bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-main/starcoin-1/starcoin/sc-data/%s --dst s3://main.starcoin.org/main/%s >> ./sync_block_main.log &'" % (
            file_compress_name, file_compress_name)
        os.system(cp_file_tar_cmd)

        # cp block list tar to s3
        # cp_blocklist_tar_cmd = "timeout 30 bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-main/starcoin-1/starcoin/sc-data/%s --dst s3://main.starcoin.org/main/%s >> ./sync_block_main.log &'" % (
        #     block_list_file_tar_name, block_list_file_tar_name)
        # os.system(cp_blocklist_tar_cmd)

        # update the last_export_height
        os.system("echo %s > ./last_export_height.txt" % end)
        os.system("aws s3api put-object --bucket main.starcoin.org --key main/last_export_height.txt --body ./last_export_height.txt")


if __name__ == "__main__":
    check_or_do()
