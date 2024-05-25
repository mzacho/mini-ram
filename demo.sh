##### Demonstrate correctness of compress

# Generate new random message
MSG=$(head -c10 /dev/random | base64); echo $MSG
# Expected output
echo -n $MSG | sha256sum
# Actual output
cargo run -- --run compress --arg $MSG --t 3809


##### Demonstrate verify_compress

# Generate mac
MAC=$(echo -n $MSG | sha256sum | cut -d ' ' -f 1); echo $MAC
