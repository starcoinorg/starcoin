# sample scripts for [wrk](https://github.com/wg/wrk) HTTP benchmarking tool

We may need to install `wrk` first. see https://github.com/wg/wrk/blob/master/INSTALL for installation guide

## command example

following commands can be used to benchmark local starcoin node

---

`wrk -t10 -c200 -d30s -s './scripts/wrk/post-list-resource.lua' http://localhost:9850`

This runs a benchmark for 30 seconds, using 10 threads, and keeping 200 HTTP connections open,
  using the script as specified by `-s`

  Output:

		...
		thread 10 created
		Running 30s test @ http://localhost:9850
		10 threads and 200 connections
		Thread Stats   Avg      Stdev     Max   +/- Stdev
			Latency   929.36ms  509.35ms   1.98s    49.79%
			Req/Sec    11.20      7.74    70.00     69.04%
		5551 requests in 30.07s, 3.89GB read
		Socket errors: connect 0, read 0, write 0, timeout 464
		Requests/sec:    184.59
		Transfer/sec:    132.32MB
		thread 1 made 626 requests and got 606 responses
		... 

---

test with 1 thread and 1 connection, could be used for api latency test

  `wrk -t1 -c1 -s './scripts/wrk/post-list-resource.lua' http://localhost:9850`

  Output:


		thread 1 created
		Running 10s test @ http://localhost:9850
		1 threads and 1 connections
		Thread Stats   Avg      Stdev     Max   +/- Stdev
			Latency   143.70ms    9.27ms 175.10ms   73.91%
			Req/Sec     7.74      2.52    10.00     55.07%
		69 requests in 10.01s, 49.46MB read
		Requests/sec:      6.89
		Transfer/sec:      4.94MB
		thread 1 made 71 requests and got 69 responses

