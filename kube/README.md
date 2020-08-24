Depoly starcoin on kubernetes

## Prepare the config and secret
	1. Create a configmap as configuration. eg: config/config-do.yaml
	   kubectl apply -f config/config-do.yaml
	2. Create a secret for node keys

## Depoly the nodes
	kubectl apply -f manifest

## Get nodes info
	./opt/node_info.sh

## Attach to some nodes
	kubectl exec  starcoin-0 --stdin --tty -- /starcoin/starcoin -n proxima -d /data/ console

## Update starcoin image version
	1. edit the container image version in manifest/starcoin.yaml
	2. kubectl apply -f manifest
