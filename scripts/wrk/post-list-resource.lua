-- example HTTP POST script which demonstrates setting the
-- HTTP method, body, and adding a header

wrk.method = "POST"
-- wrk.body   = '{"method":"state.list_resource","params":["0x1",{"decode": true }],"id":53,"jsonrpc":"2.0"}'
wrk.body = '{"method":"state.list_resource","params":["0x1",{"decode": true, "start_index": 100, "count": 20}],"id":53,"jsonrpc":"2.0"}'
-- wrk.body = '{"method":"state.list_resource","params":["0x1",{"decode": true, "start_index": 0, "count": 20}],"id":53,"jsonrpc":"2.0"}'
-- wrk.body   = '{"method":"state.list_resource","params":["0x1",{"decode": true }],"id":53,"jsonrpc":"2.0"}'
-- wrk.body = '{"method":"state.list_resource","params":["0x2b547120e3C36DEAC1d6c93dca8C4D40",{"decode": true, "start_index": 0, "count": 20}],"id":53,"jsonrpc":"2.0"}'
-- wrk.body = '{"method":"state.list_resource","params":["0x2b547120e3C36DEAC1d6c93dca8C4D40",{"decode": true, "start_index": 0, "count": 20}],"id":53,"jsonrpc":"2.0"}'
-- wrk.body = '{"method":"state.list_resource","params":["0x2b547120e3C36DEAC1d6c93dca8C4D40",{"decode": true, "start_index": 0, "count": 20}],"id":53,"jsonrpc":"2.0"}'
wrk.headers["Content-Type"] = "application/json"

local counter = 1
local threads = {}

function setup(thread)
   thread:set("id", counter)
   table.insert(threads, thread)
   counter = counter + 1
end

function init(args)
   requests  = 0
   responses = 0

   local msg = "thread %d created"
   print(msg:format(id))
end

function request()
   requests = requests + 1
   return wrk.request()
end

function response(status, headers, body)
   responses = responses + 1
   -- local msg = "response is: %s\n"
   -- print(msg:format(body))
end

function done(summary, latency, requests)
   for index, thread in ipairs(threads) do
      local id        = thread:get("id")
      local requests  = thread:get("requests")
      local responses = thread:get("responses")
      local msg = "thread %d made %d requests and got %d responses"
      print(msg:format(id, requests, responses))
   end
end
