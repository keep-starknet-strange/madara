#!/bin/bash
# exit on first error
set -e

# First argument is the number of contracts you need to generate

# WARNING: This script requires scarb 2.3.1

END=$(($1-1))

SCARB_STARKNET_DEPENDENCY="starknet = \"2.3.1\"\n[[target.starknet-contract]]\ncasm=true"

for i in $(seq 0 $END); 
do 
	dirname="counter${i}"
	libpath="${dirname}/src/lib.cairo"
	mkdir -p ${dirname}
	cd ${dirname}
	scarb init
	rm src/lib.cairo
	libpath="./src/lib.cairo"
	cp ../counter.cairo ${libpath}
	perl -i -pe "s/(?<=\b)balance(?=\b)/balance_${i}/g" "${libpath}"
	echo -e ${SCARB_STARKNET_DEPENDENCY} >> "Scarb.toml"
	scarb build
	mv target/dev/counter${i}_Counter.compiled_contract_class.json ./counter${i}.compiled_contract_class.json
	mv target/dev/counter${i}_Counter.contract_class.json ./counter${i}.contract_class.json
	mv src/lib.cairo ./counter${i}.cairo
	rm -rf src/ target/ .gitignore Scarb.toml .git Scarb.lock
	cd ..
done