#! /bin/bash

echo "Updating baseline...You must have already run check_coverage.bash in order for this to work"
cargo run --bin coverage-summaries -- --summarize-functions -t trace.mvcov -s ../stdlib/compiled/latest/stdlib -o ./baseline/coverage_report
echo "DONE"
