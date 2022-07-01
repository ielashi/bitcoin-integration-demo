import { Actor, AnonymousIdentity, HttpAgent } from "@dfinity/agent";

const webapp_id = process.env.BITCOIN_WALLET_CANISTER_ID;

// The interface of the Bitcoin wallet canister.
const webapp_idl = ({ IDL }) => {
  return IDL.Service({
    get_balance: IDL.Func([IDL.Text], [IDL.Nat64], ["update"]),
    get_p2pkh_address: IDL.Func([], [IDL.Text], ["update"]),
    get_utxos: IDL.Func([IDL.Text], [
      IDL.Record({
        'tip_height' : IDL.Nat32,
        'tip_block_hash' : IDL.Vec(IDL.Nat8),
        'utxos': IDL.Vec(IDL.Record({
          'outpoint': IDL.Record({
            'txid': IDL.Vec(IDL.Nat8),
            'vout': IDL.Nat32
          }),
          'value': IDL.Nat64,
          'height': IDL.Nat32
        }))
      })
    ], ["update"]),
    send: IDL.Func([IDL.Text], [], ["update"]),
  });
};

// Returns an actor that we use to call the servie methods.
export async function getWebApp() {
  // Using the identity obtained from the auth client, we can create an agent to interact with the IC.
  const agent = new HttpAgent({ identity: new AnonymousIdentity() });
  if(process.env.DFX_NETWORK === "local") {
    await agent.fetchRootKey();
  }

  // Using the interface description of our webapp, we create an actor that we use to call the service methods.
  return Actor.createActor(webapp_idl, {
    agent,
    canisterId: webapp_id,
  });
}
