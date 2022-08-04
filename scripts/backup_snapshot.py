import os
import sys
import argparse


def check_or_do(network):

    # back up last snapshot cp to backdir
    # do the increment snapshot export
    # check the snapshot is ok or not with the manifest file
    # if not, recover the backdir, then exit.
    # if ok, rm backup dir, and do the next step

    # check if backup exists, delete it
    if_backup_exists_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'if [ -d /sc-data/snapshotbak ]; then rm -rf /sc-data/snapshotbak;fi'" % network
    )
    if if_backup_exists_status != 0:
        print("check if backup exists then delete failed, if_backup_exists_status:%s" %
              if_backup_exists_status)
        sys.exit(1)
    print("check if backup exists then delete succeeded")

    backup_snapshot_status = os.system(
        "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'cp -r /sc-data/snapshot /sc-data/snapshotbak'" % network)
    if backup_snapshot_status != 0:
        print("backup snapshot failed, backup_snapshot_status:%s" %
              backup_snapshot_status)
        sys.exit(1)
    print("backup snapshot succeeded")


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
