#!/bin/bash
kubectl patch storageclass alicloud-disk-available -p '{"metadata": {"annotations":{"storageclass.kubernetes.io/is-default-class":"true"}}}'
