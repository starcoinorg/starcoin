# Stratum mining protocol

JSON-RPC 2.0 over TCP. Messages are newline-delimited (`\n`).
The server does not validate `jsonrpc` on requests, so it can be omitted by miners.

## Field formats
- `id` (worker id): 4 bytes hex string (8 hex chars).
- `job_id`: 8 bytes hex string (16 hex chars), `sha3_256(minting_blob)[0..8]`.
- `target`: 8 bytes little-endian hex string (16 hex chars).
- `blob`: 76 bytes hex string (152 hex chars). Bytes `[35..39]` are the worker id.
- `nonce`: hex string up to 8 chars (4 bytes). Miners typically send little-endian.
- `result`: optional hex string, accepted but not validated by server.

## login

Miner send `login` request after connection successfully established for authorization on pool.

#### Example request:
```json
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "login",
  "params": {
    "login": "fikgol",
    "pass": "123456",
    "agent": "XMRig/2.6.4 (Linux i686) libuv/1.22.1-dev gcc/4.7.3",
    "algo": ["cn", "cn/1", "cn/0", "cn/xtl", "cn/msr", "cn/xao", "cn/rto"]
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
    "id": "265ba42b",
    "job": {
      "blob": "dee268112f94469493d85826a6ef18ab738f915bffc10aa2ac4cce7556f0eefd000000265ba42b00000000000000000000000000000000000000000000000000000000000000000000000001",
      "job_id": "153b266775317b08",
      "id": "265ba42b",
      "target": "f753e3a59bc42000",
      "height": 0
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
    "blob": "f31049c3a5b3f50d19b80cda63ab7f26ec052e12391dfce0fb19d9dcda6187e3000000265ba42b00000000000000000000000000000000000000000000000000000000000000000000000001",
    "job_id": "05c85fa3d95a6037",
    "id": "265ba42b",
    "target": "f753e3a59bc42000",
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
      "id": "265ba42b",
      "job_id": "05c85fa3d95a6037",
      "nonce": "07000005",
      "result": "02c6968d5517262ad286678c1b4adb5680ac0c83c5a5b6bab5e29f109acd6f01"
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
    "id": "265ba42b"
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

## logout
Miner send `logout` to close the session.
#### Example request:
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "method": "logout",
  "params": {
    "id": "265ba42b"
  }
}
```

#### Example success reply:
```json
{
  "id": 3,
  "jsonrpc": "2.0",
  "error": null,
  "result": false
}
```
