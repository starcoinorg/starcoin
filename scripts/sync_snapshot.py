import os
import sys
import argparse


def check_or_do(network):

    # do the increment snapshot export
    export_snapshot_cmd = "kubectl exec -it -n starcoin-%s starcoin-1 -- /starcoin/starcoin_db_exporter export-snapshot --db-path /sc-data/%s -n %s -o /sc-data/snapshot -t true" % (
        network, network, network)
    export_snapshot_status = os.system(export_snapshot_cmd)
    if export_snapshot_status != 0:
        print("export snapshot failed, export_snapshot_status:%s" %
              export_snapshot_status)
        os.system(
            "kubectl exec -it -n starcoin-%s starcoin-1 -- bash -c 'rm -rf /sc-data/snapshot;mv /sc-data/snapshotbak /sc-data/snapshot'" % network)
        sys.exit(1)
    print("export snapshot succeeded")


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
