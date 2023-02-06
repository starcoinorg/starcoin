import os
import time

from datetime import datetime


def trigger_scale():
    # get last scaler pod name
    last_pod = ""
    get_last_scaler_pod_cmd = "wget https://s3.ap-northeast-1.amazonaws.com/main1.starcoin.org/main/last_scaler_pod.txt -O ./last_scaler_pod.txt"
    os.system(get_last_scaler_pod_cmd)
    with open('./last_scaler_pod.txt', 'r') as f:
        last_pod = f.read().strip()

    os.system(
        "kubectl get pods -n kube-system | grep auto | awk -F ' ' '{print$1}' > tmp")
    with open('tmp', 'r') as f:
        lastest_scalerpod = f.read().strip()
    print("current time is %s, last_pod is %s, lastest_scalerpod is %s" % (
        datetime.now().strftime("%Y-%m-%d, %H:%M:%S"), last_pod, lastest_scalerpod))

    if lastest_scalerpod != last_pod and lastest_scalerpod != "":
        print("current time is %s, last_pod is %s, lastest_scalerpod is %s, we will trigger asg to scale one node up" % (
            datetime.now().strftime("%Y-%m-%d, %H:%M:%S"), last_pod, lastest_scalerpod))
        os.system(
            "eksctl scale nodegroup --region=ap-northeast-1 --cluster=starcoin2 --nodes=1 ci-pool-200G")

        # update last_scaler_pod.txt
        last_pod = lastest_scalerpod
        os.system("echo %s > ./last_scaler_pod.txt" % lastest_scalerpod)
        os.system("aws s3api put-object --bucket main1.starcoin.org --key main/last_scaler_pod.txt --body ./last_scaler_pod.txt")


if __name__ == "__main__":
    trigger_scale()
