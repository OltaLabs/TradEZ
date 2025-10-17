rm -rf ~/.tradez-node
rm -rf ~/.tezos-client/NetXdQprcVkpa*
rm -rf ~/.tezos-client/smart_rollups

octez-node config init --network "sandbox" --rpc-addr localhost --connections 0 --data-dir ~/.tradez-node

octez-node identity generate --data-dir ~/.tradez-node

octez-node run --history-mode full:100 --synchronisation-threshold 0 --network "sandbox"  --data-dir ~/.tradez-node
