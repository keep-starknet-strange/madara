#!/bin/bash
# exit on first error
set -e

# First argument is the number of contracts you need to generate

END=$1

SCARB_STARKNET_DEPENDENCY="starknet = \"2.1.0\"\n[[target.starknet-contract]]\ncasm=true"

for i in $(seq 0 $END); 
do 
	dirname="Counter${i}"
	filepath="${dirname}/src/lib.cairo"
	mkdir -p ${dirname}
	cd ${dirname}
	filepath="src/lib.cairo"
	scarb init
	rm src/lib.cairo
	cp ../Counter.cairo ${filepath}
	sed -i '' -e "s/Counter/Counter${i}/g" ${filepath}
	sed -i '' -e "s/balance/balance_${i}/g" ${filepath}
	sed -i '' -e "s/+ amount/+ amount + ${i} + 1/g" ${filepath}
	echo -e ${SCARB_STARKNET_DEPENDENCY} >> "Scarb.toml"
	scarb build
	mv target/dev/Counter${i}_Counter${i}.casm.json ./Counter${i}.casm.json
	mv target/dev/Counter${i}_Counter${i}.sierra.json ./Counter${i}.sierra.json
	mv src/lib.cairo ./Counter${i}.cairo
	rm -rf src/ target/ .gitignore Scarb.toml .git
	cd ..
done