import { Actor, AnonymousIdentity, HttpAgent } from "@dfinity/agent";

const webapp_id = process.env.BITCOIN_WALLET_CANISTER_ID;

// The interface of the Bitcoin wallet canister.
const webapp_idl = ({ IDL }) => {
  return IDL.Service({
    get_balance: IDL.Func([IDL.Text], [IDL.Nat64], ["update"]),
    get_p2pkh_address: IDL.Func([], [IDL.Text], ["update"])
  });
};

const init = ({ IDL }) => {
  return [];
};

// Autofills the II Url to point to the correct canister.
export const iiUrl =
  (process.env.DFX_NETWORK === "local") ?
    `http://localhost:8000/?canisterId=${process.env.II_CANISTER_ID}` : (
  (process.env.DFX_NETWORK === "ic") ?
    `https://${process.env.II_CANISTER_ID}.ic0.app` :
    `https://${process.env.II_CANISTER_ID}.dfinity.network`
);

// Redirects the user to another webpage.
export function redirectTo(page = "") {
  var url = new URL(document.location.href);
  window.location.replace((page !== "" ? page + ".html" : "") + url.search);
}

// Redirects the user to the dashboard weboage.
export function redirectToDashboard() {
  redirectTo("dashboard");
}

// Redirects the user to the login webpage.
export function redirectToLogin() {
  redirectTo();
}

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
