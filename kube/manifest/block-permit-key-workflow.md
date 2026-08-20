# Block-permit signer key workflow

The Halley, Barnard, and Main signer keys are independent Ed25519 keys. Their
authentication keys and activation heights are consensus release inputs in
`types/src/block_permit.rs`; private keys must never enter Git, an image, a
ConfigMap, a command-line value, or a pod environment variable.

The frozen generation-1 release inputs are:

| Network | Activation height | Authentication key |
| --- | ---: | --- |
| Halley | 2,894,400 | `0x157b42a66560e83144deecdb43d5bac70a5e116b0938d277192de7d01617e2ac` |
| Barnard | 19,667,300 | `0xca72d0ee15cf78c1232218c8d4c241c87e6cdc6953fea813c7be2191f9388e67` |
| Main | 32,165,300 | `0x6ea544078efcbac81e9d6197090b1a106cd69f2fef83204a2ddecf05980601cd` |

## Generate and freeze release inputs

Generate every key on a trusted operator host with a release-built Starcoin
binary. Keep the shell quiet and make all plaintext files owner-readable only:

```bash
umask 077
install -d -m 0700 /secure/path/block-permit-g1
for network in halley barnard main; do
  target/release/starcoin -o json account generate-keypair \
    > "/secure/path/block-permit-g1/${network}.json"
  jq -er '.ok[0].private_key' \
    "/secure/path/block-permit-g1/${network}.json" \
    > "/secure/path/block-permit-g1/${network}.private-key"
  chmod 0600 "/secure/path/block-permit-g1/${network}.private-key"
done
```

Copy only `.ok[0].auth_key` from each JSON document into the matching release
constant. Record `.ok[0].public_key`, the authentication key, activation height,
release version, and Git commit in an operator-visible release record. Confirm
the three authentication keys are distinct before building the artifact.

Encrypt the complete record immediately to at least one audited operator
recipient, verify that it decrypts, and then remove the plaintext directory:

```bash
tar -C /secure/path -cf - block-permit-g1 \
  | age -R /secure/path/operator-age-recipients.txt \
      -o /secure/backup/block-permit-g1.tar.age
age --decrypt -i /secure/path/operator-age-identity.txt \
  /secure/backup/block-permit-g1.tar.age | tar -tf -
```

Do not print or log a decrypted payload. Transfer and retention must follow the
same access policy as production validator keys.

## Install an immutable Kubernetes secret

Use one namespace-local immutable secret per network. The annotations are public
audit data; the `private-key` data item is the encoded Starcoin Ed25519 private
key file consumed by `--block-permit-private-key-file`.

```bash
network=halley
namespace=starcoin-halley
activation_height=2894400
authentication_key=0x157b42a66560e83144deecdb43d5bac70a5e116b0938d277192de7d01617e2ac

kubectl -n "$namespace" create secret generic block-permit-signer-g1 \
  --from-file=private-key="/secure/path/block-permit-g1/${network}.private-key" \
  --dry-run=client -o json \
  | jq --arg network "$network" \
       --arg height "$activation_height" \
       --arg auth "$authentication_key" \
       '.immutable = true
        | .metadata.annotations = {
            "starcoin.org/network": $network,
            "starcoin.org/activation-height": $height,
            "starcoin.org/authentication-key": $auth
          }' \
  | kubectl create -f -
```

Repeat with the fixed Barnard and Main release inputs. Never reuse a Secret
object or key across namespaces. A rotation creates a new key, a new secret
generation such as `block-permit-signer-g2`, and a new compiled release. Secret
data is never edited in place. If the pre-activation schedule changes while the
key stays fixed, patch only the public activation/release annotations and verify
that a hash of the `private-key` data item is identical before and after.

## Mount only where block templates are signed

The Halley and Barnard manifests mount the secret read-only with mode `0400` on
their single block-template node. The Main StatefulSet has multiple seed nodes,
so its namespace Secret must remain unmounted until a single Main template
signer workload is selected. Do not add the Main secret to the shared seed-node
pod template.

Main is a single-writer deployment, not an active/standby signer pair:

- `starcoin-main-block-permit-signer` must have exactly one replica. Its startup
  command must refuse to run unless `POD_NAME` is
  `starcoin-main-block-permit-signer-0`.
- The pool-facing Service must select both the signer application label and
  `statefulset.kubernetes.io/pod-name: starcoin-main-block-permit-signer-0`.
  Before routing pool traffic, its EndpointSlice must contain exactly one ready,
  non-terminating endpoint.
- Never put multiple Starcoin nodes behind the pool upstream. Mining jobs are
  process-local; a template obtained from one node can be rejected as
  `TaskMisMatch` or `TaskEmpty` by another node.
- Override `NODE_RPC_URL` only on the ASIC and CPU stratum StatefulSets, using
  `ws://starcoin-main-block-permit-signer.starcoin-main.svc.cluster.local:9870`.
  Do not change the shared mining-pool ConfigMap: pool, payout, and unrelated
  workloads do not need the signer endpoint or a restart.
- A warm recovery node may synchronize with no private-key volume, no signer
  Service label, and no path from the pool. It must not be promoted
  automatically.
- Normal restart and node rescheduling use the same replica and ReadWriteOnce
  PVC. For manual recovery, first fence the old signer and confirm that the
  Service has zero endpoints and the old PVC is detached. Only then may the key
  be mounted on the recovery workload. Restart every stratum process after the
  Service again has exactly one ready endpoint so no process-local job survives
  the cutover.

Do not test signer failover on Main. Exercise restart and adversarial block
rejection on Halley; on Main, use read-only preflight checks and fail closed.

Safe audit commands inspect only metadata:

```bash
kubectl -n starcoin-halley get secret block-permit-signer-g1 \
  -o jsonpath='{.immutable}{"\t"}{.metadata.annotations}{"\n"}'
kubectl -n starcoin-halley get pod starcoin-0 \
  -o jsonpath='{.spec.containers[0].volumeMounts}{"\n"}'
```

Before activation, verify the image digest, mounted file mode, compiled network
authentication key, signer/key match, node synchronization, and a fresh PVC
snapshot. After activation, verify permit envelopes on every sampled canonical
block and confirm that an unsigned or incorrectly signed block is rejected.
