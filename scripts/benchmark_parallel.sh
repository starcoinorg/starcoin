
#RUST_LOG=info cargo bench --features fuzzing -p 'starcoin-block-executor'
#RUST_LOG=info cargo bench --features fuzzing -p 'starcoin-transaction-benchmarks'

STARCOIN_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && cd .. && pwd)"
TXN_NUMS=5000,10000,50000,100000,500000
ACCOUNT_NUMS=1000,10000,100000

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    physical_cores=$(lscpu -p=CPU,Core,Socket | grep -v '#' | sort -u -t, -k2,3 | wc -l)
elif [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    physical_cores=$(sysctl -n hw.physicalcpu)
else
    echo "Unknown OS"
    exit 1
fi

# 创建一个数组来存储每个核心的模 2 值
declare -a power_of_two_array

if (( physical_cores <= 4 )); then
    power_of_two_array=(1 2 4)
else
  current_power=4
  while (( current_power <= physical_cores )); do
      power_of_two_array+=($current_power)
      current_power=$((current_power << 1))
  done

  if ! [[ " ${power_of_two_array[@]} " =~ " ${physical_cores} " ]]; then
      power_of_two_array+=($physical_cores)
  fi
fi

# 打印数组
IFS=','
power_of_two_str="${power_of_two_array[*]}"

#echo "Power of two array: ${power_of_two_str[@]}"
eval RUST_LOG=info cargo run --release -p "starcoin-transaction-benchmarks" --features fuzzing -- --concurrency-level "$power_of_two_str" --txn-nums "$TXN_NUMS" --account-nums="$ACCOUNT_NUMS"