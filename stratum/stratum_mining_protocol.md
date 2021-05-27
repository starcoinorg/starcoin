# Stratum mining protocol
## login

Miner send `login` request after connection successfully established for authorization on pool.

#### Example request:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "login",
  "params": {
    "login": "48edfHu7V9Z84YzzMa6fUueoELZ9ZRXq9VetWzYGzKt52XU5xvqgzYnDK9URnRoJMk1j8nLwEVsaSWJ4fhdUyZijBGUicoD",
    "pass": "x",
    "agent": "Ibctminer/1.0.0"
  }
}
```

#### Example success reply:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "error": null,
  "result": {
    "id": "1be0b7b6-b15a-47be-a17d-46b2911cf7d0", // the id of the working miner
    "job": {
      "blob": "070780e6b9d60586ba419a0c224e3c6c3e134cc45c4fa04d8ee2d91c2595463c57eef0a4f0796c000000002fcc4d62fa6c77e76c30017c768be5c61d83ec9d3a085d524ba8053ecc3224660d",
      "job_id": "q7PLUPL25UV0z5Ij14IyMk8htXbj",
	  "id": "1be0b7b6-b15a-47be-a17d-46b2911cf7d0", //the id of the working miner
      "target": "b88d0600",
	  "height": 0, // height must always be 0
    },
    "status": "OK"
  }
}
```

#### Example error reply:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "error": {
    "code": -1,
    "message": "Invalid payment address provided"
  }
}
```

## job
Pool send new job to miner. Miner should switch to new job as fast as possible.

#### Example notification:
```json
{
  "jsonrpc": "2.0",
  "method": "job",
  "params": {
    "blob": "0707d5efb9d6057e95a35f868231780b3a8649c4e57f3c77eaf437329243eef0b9f4b6987d05b900000000cae7754cb85a0ad8eebf3e0bf55f3ec5e754a1d6b05d46e5c358f907dbcbb72b01",
    "job_id": "4BiGm3/RgGQzgkTI/xV0smdA+EGZ",
    "target": "b88d0600"
	"height": 0
  }
}
```

## submit
Miner send `submit` request after share was found.

#### Example request:
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "method": "submit",
  "params": {
    "id": "1be0b7b6-b15a-47be-a17d-46b2911cf7d0",
    "job_id": "4BiGm3/RgGQzgkTI/xV0smdA+EGZ",
    "nonce": "d0030040",
    "result": "e1364b8782719d7683e2ccd3d8f724bc59dfa780a9e960e7c0e0046acdb40100"
  }
}
```

#### Example success reply:
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "error": null,
  "result": {
    "status": "OK"
  }
}
```

#### Example error reply:
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "error": {
    "code": -1,
    "message": "Low difficulty share"
  }
}
```

## keepalived
Miner send `keepalived` to prevent connection timeout.
#### Example request:
```json
{
  "id": 2,
  "method": "keepalived",
  "params": {
    "id": "1be0b7b6-b15a-47be-a17d-46b2911cf7d0" //the id of the working miner
  }
}
```

#### Example success reply:
```json
{
  "id": 2,
  "jsonrpc": "2.0",
  "error": null,
  "result": {
    "status": "KEEPALIVED"
  }
}
```
