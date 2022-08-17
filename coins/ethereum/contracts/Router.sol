//SPDX-License-Identifier: AGPLv3
pragma solidity ^0.8.0;

import "./Schnorr.sol";

contract Router is Schnorr {
    // contract owner
    address owner;

    // nonce is incremented for each batch of transactions executed
    uint256 nonce; 

    struct PublicKey {
        uint8 parity;
        bytes32 px;
    }

    // current aggregated validator public key 
    PublicKey publicKey;

    struct Transaction {
        address to;
        uint256 value;
        bytes data;
    }

    struct Signature {
        bytes32 e;
        bytes32 s;
    }

    event Executed(bool success);

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        require(msg.sender == owner, "must be called by the contract owner");
        _;
    }

    function setPublicKey(
        PublicKey memory _publicKey
    ) public onlyOwner {
        publicKey.px = _publicKey;
    }

    function updatePublicKey(
        PublicKey memory _publicKey,
        Signature memory sig
    ) public {
        bytes32 message = keccak256(abi.encode(_publicKey.parity, _publicKey.px));
        require(verify(publicKey.parity, publicKey.px, message, sig.s, sig.e), "failed to verify signature");
        publicKey = _publicKey;
    }

    // execute accepts a list of transactions to execute as well as a Schnorr signature.
    // if signature verification passes, the given transactions are executed.
    // if signature verification fails, this function will revert.
    // if any of the executed transactions fail, this function will return false but *not* revert.
    // if all the executed transactions succeed, this function returns true.
    function execute(
        Transaction[] calldata transactions, 
        Signature memory sig
    ) public returns (bool) {
        bytes32 message = keccak256(abi.encode(nonce, transactions));
        require(verify(parity, px, message, sig.s, sig.e), "failed to verify signature");
        nonce++;
        bool allOk = true;
        for(uint256 i = 0; i < transactions.length; i++) {
                (bool success, ) = transactions[i].to.call{value: transactions[i].value}(
                    transactions[i].data
                );
                emit Executed(success);
                allOk = success && allOk;
        }
        return allOk;
    }
}