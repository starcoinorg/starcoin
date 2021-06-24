## Suggested steps for updating mapping

### step1:
 Update the relevant templates or settings, Use indexer (may require program compatibility) to re-run and write
 to the new index, you can modify the startup parameter {--es-index-prefix}, such as 0527barnard, and it is
 recommended to adjust the --bulk-size to shorten the processing time.

### step2:

 First, check if the index in the previous step catches up with the latest data.

### step3:

 Stop the old indexer and back up the online index.

``` 
    POST /barnard.blocks/_open
    
    PUT /barnard.blocks/_settings
    {
      "settings": {
        "index.blocks.write": "true"
      }
    }
    
    POST /barnard.blocks/_clone/barnard.blocks_0527
    {
      "settings": {
        "index.blocks.write": null
      }
    }
    
    POST /barnard.txn_infos/_open
    PUT /barnard.txn_infos/_settings
    {
      "settings": {
        "index.blocks.write": "true"
      }
    }
    
    POST /barnard.txn_infos/_clone/barnard.txn_infos_0527
    {
      "settings": {
        "index.blocks.write": null
      }
    }
    
 ``` 
### step4:
    
Check the index status of the backup, green is normal. And delete the index on the line.

``` 
   Check:
   GET /_cluster/health/barnard.blocks_0527?wait_for_status=green&timeout=30s
   GET /_cluster/health/barnard.txn_infos_0527?wait_for_status=green&timeout=30s

   DELETE /barnard.txn_infos
   DELETE /barnard.blocks
 ``` 
### step5:
   
Stop the new indexer from being written and clone the new index.

``` 
   POST /0527barnard.blocks/_open

   PUT /0527barnard.blocks/_settings
   {
     "settings": {
       "index.blocks.write": "true"
     }
   }
     POST /0527barnard.blocks/_clone/barnard.blocks
     {
       "settings": {
         "index.blocks.write": null
       }
     }

   POST /0527barnard.txn_infos/_open
   PUT /0527barnard.txn_infos/_settings
   {
     "settings": {
       "index.blocks.write": "true"
     }
   }

     POST /0527barnard.txn_infos/_clone/barnard.txn_infos
     {
       "settings": {
         "index.blocks.write": null
       }
     }
``` 
### step6:
    Start a new indexer, restore the index appending, and check if the business is normal.


### Other issues：

Possible failure of the above command execution.
1,did not follow the steps, for example: no open set setting will not succeed, the following clone will fail
2, the execution of the clone was interrupted by the snapshot operation, don't worry, wait for another execution, 
snapshot generally lasts less than a minute.



