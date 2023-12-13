const Api = require('@polkadot/api')
const { Keyring } = require('@polkadot/keyring');
const { hexToU8a, u8aToHex } = require('@polkadot/util');
const { blake2AsU8a, blake2AsHex, cryptoWaitReady } = require('@polkadot/util-crypto');

const { ApiPromise, WsProvider } = Api

const keyring = new Keyring({ type: 'sr25519', ss58Format: 42 });
await cryptoWaitReady();

const alice = keyring.addFromUri('//Alice');
const bob = keyring.addFromUri('//Bob');

const api = await ApiPromise.create({ provider: new WsProvider('ws://127.0.0.1:9944') });

// WRITE OPERATIONS:

const hashName = (owner, name, asHex = true) => {
    const constantBytes = new TextEncoder().encode('modlpy/paych____');
    let nameBytes = api.registry.createType('Vec<u8>', name).toU8a();
    const message = new Uint8Array([...constantBytes, ...owner.publicKey, ...nameBytes]);
    if (asHex) return blake2AsHex(message);
    return blake2AsU8a(message);
}

// My Organization: 0x14defea6f8a3e4f641e80c43d62b81825e0388ed0b72c1a049a31babb6f493af
hashName(alice, "My Organization")
// My Service:      0xa5309aeb197bd0f5ddc06af2888f06cf3ed1d384bc1f9281f9b45a6d76f29936
hashName(alice, "My Service")
// Huge Name:       0x2987b056c5bf7290246aecf8694bf5a1990fe081f587658643d895a72dac3c1e
hashName(alice, " 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789")

const hashChannelId = (owner, organizationIdBytes, serviceIdBytes, asHex = true) => {
    const constantBytes = new TextEncoder().encode('modlpy/paych____');
    const message = new Uint8Array([...constantBytes, ...owner.publicKey, ...organizationIdBytes, ...serviceIdBytes]);
    if (asHex) return blake2AsHex(message);
    return blake2AsU8a(message);
}

// Alice creates an Organization
let xtOrg = (await api.tx.paymentChannels.createOrganization("My Organization", [], "").signAndSend(alice)).toString();

const organizationId = hashName(alice, "My Organization", false);

// Alice creates a Service under her Organization
let xtSrv = (
    await api.tx.paymentChannels.createService(
        [alice.address, organizationId],
        "My Service",
        1_000_000_000_000,
        50,
        50,
        3,
        "",
    ).signAndSend(alice)
).toString();

const serviceId = hashName(alice, "My Service", false);

// Bob opens a Channel to Alice's Service
let xtChn = (
    await api.tx.paymentChannels.openChannel(
        [[alice.address, organizationId], serviceId],
        100,
    ).signAndSend(bob)
).toString();

const bobChannelId = hashChannelId(bob, organizationId, serviceId, false);

// READ OPERATIONS
let aliceOrganizations = [];
let queryOrgStorage = await api.query.paymentChannels.organizations.entries(alice.address);
queryOrgStorage.forEach(([{ args: [owner, id] }, organization]) => {
    console.log(`\n(OrgOwner, OrgId): (${owner}, ${id})\n\n${JSON.stringify(organization, null, 2)}`);
    aliceOrganizations.push(organization.toHuman());
});

let aliceOrgServices = [];
let querySrvStorage = await api.query.paymentChannels.services.entries(aliceOrganizations[0].id);
querySrvStorage.forEach(([{ args: [orgId, serviceId] }, service]) => {
    console.log(`\n(OrgId, SrvId): (${orgId}, ${serviceId})\n\n${JSON.stringify(service, null, 2)}`);
    aliceOrgServices.push(service.toHuman());
});

let bobChannels = [];
let queryChnStorage = await api.query.paymentChannels.channels.entries(bob.address);
queryChnStorage.forEach(([{ args: [owner, id] }, channel]) => {
    console.log(`\n(owner, channelId): (${owner}, ${id})\n\n${JSON.stringify(channel, null, 2)}`);
    bobChannels.push(channel.toHuman());
});

// Signature and Claiming

// (b"modlpy/paych____", bobChannelId, 1)
// 0x6e2475de30f1a91bac99b8536165a8361084c2fc7f2fbda55bcfd66103e81405158342810c2c71f3806996c081a3423ad1c0cd72dd0b25b04042eadf4ab3aa88
// 0x1e137e8426f5298d9b33a1be88df90cc339b2a5f0001529ea5f2b2b022b8983970954e553b5cf440eeb08f8da22766f81112672ee0a2ff2c18a8c22de957ad83

// (b"modlpy/paych____", bobChannelId, 77)
// 0x32e141c6fcad235540360a1d9a5106a0bfa4c90549c414ec7956359f7f0f18732a8349943d08371d78a8eda9a44cec89021f1f3e7e3d4ee8bc8be9ff4f123a81
// 0xca8f0f38581976aa52cdc90893a2997e0c52c56b58b878bb5f1a8f1df7ed1608978f5c736285f0c95e018cffdf6b69a489dfe9e5a6bbb6f297704cb085e1438b

// (b"modlpy/paych____", bobChannelId, 100)
// 0x62dc66ce7e27dc266463abdd560250c6542b66c197722e840e797f968da9f86c7aef31f32a53f019229fe5c4d6620fc444ef741c8b00a1cbd097161b1e95a28d
// 0x5a5c1c6ed37fb69e72d76d72dc3965fca6205b48f5cc3aa6419fbec52d672239b5e1ff52b984d8f2efc2f6c6c6141c232d0097c167903aaa3fe1472880d9e484

const signChannelCounter = (signer, channelId, version, counter, asHex = true) => {
    const constantBytes = new TextEncoder().encode('modlpy/paych____');
    const c = api.registry.createType('u32', counter).toU8a();
    const v = api.registry.createType('u32', version).toU8a();
    const sig = signer.sign(blake2AsU8a([...constantBytes, ...channelId, ...v, ...c]));
    if (asHex) return `0x${Buffer.from(sig).toString('hex')}`;
    return sig;
}

const bobSig = signChannelCounter(bob, bobChannelId, 1, 77, false);

// Alice claims 77 (of 100) from Bob's channel (using Sr22519)
let xtClaim = (
    await api.tx.paymentChannels.claimChannelFunds(
        [bob.address, bobChannelId],
        77,
        { sr25519: bobSig },
    ).signAndSend(alice)
).toString();
