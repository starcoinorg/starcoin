import os
import time
import sys
import argparse


def check_or_do(network):

    # check the snapshot is ok or not with the manifest file
    # if not, recover the backdir, then exit.
    # if ok, rm backup dir, and do the next step

    export_state_node_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(more /sc-data/snapshot/manifest.csv| grep -w state_node | awk -F ' ' '{print\$2}') -eq \$(wc -l /sc-data/snapshot/state_node| awk -F ' ' '{print\$1}') ]; then exit 0; else exit 1;fi\"" % network)
    export_acc_node_transaction_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(more /sc-data/snapshot/manifest.csv| grep acc_node_transaction | awk -F ' ' '{print\$2}') -eq \$(wc -l /sc-data/snapshot/acc_node_transaction| awk -F ' ' '{print\$1}') ]; then exit 0; else exit 1;fi\"" % network)
    export_acc_node_block_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(more /sc-data/snapshot/manifest.csv| grep acc_node_block | awk -F ' ' '{print\$2}') -eq \$(wc -l /sc-data/snapshot/acc_node_block| awk -F ' ' '{print\$1}') ]; then exit 0; else exit 1;fi\"" % network)
    export_block_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(more /sc-data/snapshot/manifest.csv| grep -w block | awk -F ' ' '{print\$2}') -eq \$(wc -l /sc-data/snapshot/block| awk -F ' ' '{print\$1}') ]; then exit 0; else exit 1;fi\"" % network)
    export_block_info_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(more /sc-data/snapshot/manifest.csv| grep block_info | awk -F ' ' '{print\$2}') -eq \$(wc -l /sc-data/snapshot/block_info| awk -F ' ' '{print\$1}') ]; then exit 0; else exit 1;fi\"" % network)
    export_state_node_prev_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c \"if [ \$(more /sc-data/snapshot/manifest.csv| grep state_node_prev | awk -F ' ' '{print\$2}') -eq \$(wc -l /sc-data/snapshot/state_node_prev| awk -F ' ' '{print\$1}') ]; then exit 0; else exit 1;fi\"" % network)

    if export_state_node_status != 0 or export_acc_node_transaction_status != 0 or export_acc_node_block_status != 0 or export_block_status != 0 or export_block_info_status != 0 or export_state_node_prev_status != 0:
        print("check export snapshot status, found something wrong, export_state_node_status:%s, export_acc_node_transaction_status:%s, export_acc_node_block_status:%s, export_block_status:%s, export_block_info_status:%s, export_state_node_prev_status:%s" % (
            export_state_node_status,
            export_acc_node_transaction_status,
            export_acc_node_block_status,
            export_block_status,
            export_block_info_status,
            export_state_node_prev_status))
        print("check snapshot found somethind wrong, now delete snapshot and recover with snapshotbak")
        os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'rm -rf /sc-data/snapshot;mv /sc-data/snapshotbak /sc-data/snapshot'" % network)
        sys.exit(1)
    print("check snapshot succeeded, now delete snapshotbak")
    os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'rm -rf /sc-data/snapshotbak'" % network)

    # tar snapshot
    tar_snapshot_cmd = "kubectl exec -it -n starcoin-%s starcoin-1 -- tar -czvf /sc-data/%s -C /sc-data/ %s " % (
        network, "snapshot.tar.gz", "snapshot")
    os.system(tar_snapshot_cmd)

    # cp snapshot.tar.gz to s3
    cp_snapshot_tar_cmd = "bash -c 'export AWS_REGION=ap-northeast-1;skbn cp --src k8s://starcoin-%s/starcoin-1/starcoin/sc-data/%s --dst s3://main.starcoin.org/%s/%s '" % (
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
