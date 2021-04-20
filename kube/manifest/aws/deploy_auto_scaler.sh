kubectl apply -f https://raw.githubusercontent.com/kubernetes/autoscaler/master/cluster-autoscaler/cloudprovider/aws/examples/cluster-autoscaler-autodiscover.yaml

kubectl annotate serviceaccount cluster-autoscaler \
  -n kube-system \
  eks.amazonaws.com/role-arn=arn:aws:iam::576184071779:role/eksctl-starcoin2-addon-iamserviceaccount-kub-Role1-12GL6HE7ZIAI9 --overwrite

kubectl patch deployment cluster-autoscaler \
  -n kube-system \
  -p '{"spec":{"template":{"metadata":{"annotations":{"cluster-autoscaler.kubernetes.io/safe-to-evict": "false"}}}}}'

echo "https://docs.aws.amazon.com/eks/latest/userguide/cluster-autoscaler.html"

kubectl -n kube-system edit deployment.apps/cluster-autoscaler

kubectl set image deployment cluster-autoscaler \
  -n kube-system \
  cluster-autoscaler=k8s.gcr.io/autoscaling/cluster-autoscaler:v1.19.1
