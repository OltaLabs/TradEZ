#!/bin/sh
set -x

# rm -rf ~/.tradez-node

# First, we need to configure the Octez node to run in sandbox mode:

octez-node config init --network "sandbox" --rpc-addr localhost --connections 0 --data-dir ~/.tradez-node

# The --rpc-addr option sets the TCP address for reaching the RPC server instance. 
# The --connections option removes the requirement for establishing connections with other nodes.
# Then, we generate the identity of the node we want to run:

octez-node identity generate --data-dir ~/.tradez-node

# To start the newly configured node in the background, you will need:

nohup octez-node run --synchronisation-threshold 0 --network "sandbox"  --data-dir ~/.tradez-node > node.log &

# The --synchronisation-threshold option specifies the number of peers required to complete chain synchronization 
# for bootstrapping the node. In our case, since there are no other nodes, this value is set to 0. 
# Essentially, you have created a network consisting of a single node.

# To use the octez-client command, we need to wait for the node to get bootstrapped:

while ! octez-client bootstrapped; do sleep 1; done

# When bootstrapping a new network, it is initialized with a placeholder economic protocol known as the "genesis" 
# protocol. Our goal is to upgrade the network's economic protocol to the alpha version, enabling us to originate a Smart 
# Rollup. To accomplish this, we must first create an account and use its key to activate the new protocol:

octez-client import secret key activator unencrypted:edsk31vznjHSSpGExDMHYASz45VZqXN4DPxvsa4hAyY8dHM28cZzp6

# To execute the activation command, we require sandbox parameters that are essentially modified versions of the 
# Mainnet parameters tailored for testing purposes. The purpose of these adjustments is to create the testing phase, 
# as we do not want operations to take an excessive amount of time. It is important to note that these optimizations 
# may not reflect the performance on the actual Mainnet. The necessary data for this can be found in the 
# `sandbox-parameters.json` file.

octez-client -block genesis activate protocol ProtoALphaALphaALphaALphaALphaALphaALphaALphaDdp3zK with fitness 1 and key activator and parameters /usr/local/share/tezos/alpha-parameters/sandbox-parameters.json
# The migration to the alpha protocol requires a manual baking process. In this case, we will create new test accounts 
# to facilitate this process.

octez-client import secret key bootstrap1 unencrypted:edsk3gUfUPyBSfrS9CCgmCiQsTCHGkviBDusMxDJstFtojtc1zcpsh
octez-client import secret key bootstrap2 unencrypted:edsk39qAm1fiMjgmPkw1EgQYkMzkJezLNewd7PLNHTkr6w9XA2zdfo
octez-client import secret key bootstrap3 unencrypted:edsk4ArLQgBTLWG5FJmnGnT689VKoqhXwmDPBuGx3z4cvwU9MmrPZZ
octez-client import secret key bootstrap4 unencrypted:edsk2uqQB9AY4FvioK2YMdfmyMrer5R8mGFyuaLLFfSRo8EoyNdht3
octez-client import secret key bootstrap5 unencrypted:edsk4QLrcijEffxV31gGdN2HU7UpyJjA8drFoNcmnB28n89YjPNRFm

# The activation command at the end of the `sandbox_node.sh` file needs to be manually baked.

while octez-client bake for --minimal-timestamp; do sleep 1; done

# This will continuously bake.


