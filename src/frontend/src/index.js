import { getWebApp } from './common.js';

window.onload = async () => {
  const webapp = await getWebApp();

  const address = await webapp.get_p2pkh_address();
  console.log(address);
  document.getElementById("address").innerText = address;

  console.log("bitcoin balance");
  const balance = await webapp.get_balance(address);
  console.log(balance);
  document.getElementById("balance").innerText = Number(balance) / (10 ** 8);

  const utxos = await webapp.get_utxos(address);
  console.log(utxos);
  document.getElementById("utxos").innerText = utxos;

  document.getElementById("send").onclick = async function sendBitcoin() {
    const address = prompt("Address to send 1 BTC to.");
    if (address == null) {
      alert("Address must be specified");
    } else {
      await webapp.send(address);
      console.log("transaction sent");
    }
  }
};