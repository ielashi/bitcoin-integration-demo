use sha2::Digest;

pub fn p2pkh_address_from_public_key(public_key: Vec<u8>) -> String {
    // sha256 + ripmd160
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(sha256(public_key));
    let result = hasher.finalize();

    // mainnet: 0x00, testnet: 0x6f
    let mut data_with_prefix = vec![0x6f];
    data_with_prefix.extend(result);

    //let data_with_prefix_b58 = bs58::encode(data_with_prefix);
    // TODO: get rid of clone?
    let checksum = &sha256(sha256(data_with_prefix.clone()))[..4];

    let mut full_address = data_with_prefix;
    full_address.extend(checksum);

    bs58::encode(full_address).into_string()
}

pub fn sha256(data: Vec<u8>) -> Vec<u8> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
