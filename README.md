# drug_serialization

## Project Title
drug_serialization

## Project Description
Counterfeit and re-packaged pharmaceuticals are a multi-billion dollar problem that costs lives. Existing paper-and-barcode traceability can be forged, lost, or never entered in the first place, so regulators, distributors, and patients have no reliable way to tell a genuine pack from a fake one at the point of dispense. `drug_serialization` is a Soroban smart contract that issues every drug pack a unique GTIN-like serial at production time, records every custody hop from manufacturer to distributor to pharmacist, and consumes the serial on final dispense. A second dispense attempt with the same serial is rejected on-chain, turning double-dispense into a tamper-evident counterfeit signal anyone can verify.

## Project Vision
Our vision is a global, open pharmaceutical supply chain where the authenticity and provenance of every single drug pack is verifiable by anyone with a smartphone, in any country, without trusting the manufacturer, the distributor, the pharmacist, or any single authority. By anchoring pack identity, custody, dispense, and recall on Stellar's fast and cheap Soroban layer, we want to make it economically and operationally impossible for counterfeit units to survive at the point of dispense, and to give regulators a one-call tool to recall suspicious lots across the entire downstream chain in seconds.

## Key Features
- **Per-pack production registration** — `produce` lets a manufacturer bind a unique GTIN-like serial to a drug name, regulation category, batch, and expiry, all timestamped by the ledger.
- **Custody hop tracking** — `transfer` records the latest custodian and location of a pack, incrementing a `custody_count` so verifiers can see how far a pack has travelled.
- **One-time dispense** — `dispense` is intentionally one-shot: a second call on the same serial panics, which is the on-chain counterfeit alarm.
- **Public verification** — `verify` and `is_dispensed` let any party (pharmacy scanner, regulator, end user) check a serial's history without trusting a centralized database.
- **Regulator-controlled recall** — `recall` lets the authority registered in `init` mark a serial as recalled with a reason, after which no further transfer or dispense is accepted.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** supply_chain dApp — see `contracts/drug_serialization/src/lib.rs` for the full drug_serialization business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `<to be deployed on Stellar Testnet>`
- **Explorer template:** `https://stellar.expert/explorer/testnet/contract/<to`
- **Screenshot of deployed contract on Stellar Expert:**
  `_(Screenshot of the contract page on Stellar Expert will appear here after deploy.)_`


## Future Scope
- **Frontend verifier** — a lightweight web/mobile app that scans a pack's 2D data-matrix, calls `verify` / `is_dispensed` / `is_recalled`, and shows a green / red / recall badge to the pharmacist or end user.
- **IPFS / off-chain metadata** — store large drug leaflets, lab certificates, and temperature-log digests on IPFS and anchor only their content hash on-chain via `produce`, to keep the ledger small.
- **Multi-authority recall and batch-level recall** — extend `recall` to support a list of trusted authorities and to recall an entire manufacturing batch in one call.
- **Privacy-preserving patient linkage** — replace the plaintext `patient_hash` with a zero-knowledge proof so a regulator can prove a recalled pack was never dispensed to a specific patient without revealing who actually received it.
- **Mainnet rollout and GS1 alignment** — graduate from the Soroban testnet to mainnet and align the on-chain serial format with the GS1 GTIN / NTIN / SSCC standards used by the pharmaceutical industry.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `drug_serialization` (supply_chain)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
