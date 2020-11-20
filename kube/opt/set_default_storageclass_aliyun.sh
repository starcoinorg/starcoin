#!/bin/bash
kubectl patch storageclass alicloud-disk-essd -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'
