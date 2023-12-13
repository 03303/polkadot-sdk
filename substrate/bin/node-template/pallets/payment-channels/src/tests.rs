use codec::Encode;
use frame_support::assert_ok;
use sp_core::Pair;
use sp_runtime::MultiSignature;
use sp_io::hashing::blake2_256;
use crate::{mock::*, HashId, Config, NameVec, ChannelSpecs};

fn sign_message<T: Config>(pair: sp_core::sr25519::Pair, channel_id: HashId<T>, version: u32, counter: u32) -> (Vec<u8>, MultiSignature) {
    let message = (
        b"modlpy/paych____",
        channel_id,
        version,
        counter,
    ).using_encoded(blake2_256);
    let encoded_data = Encode::encode(&message);
    let signature = MultiSignature::Sr25519(pair.sign(&message));
    (encoded_data, signature)
}

#[test]
fn workflow() {
    new_test_ext().execute_with(|| {
        let (_pair_alice, alice) = get_account("//Alice");
        let (pair_bob, bob) = get_account("//Bob");

        let organization_name: NameVec<Test> = b"My Organization".to_vec().try_into().unwrap();
        assert_ok!(PaymentChannels::create_organization(
            RuntimeOrigin::signed(alice.clone()),
            organization_name.clone(),
            vec![].try_into().unwrap(),
            vec![].try_into().unwrap(),
        ));

        let organization_id = PaymentChannels::hash_name(alice.clone(), organization_name);
        let organization = (alice.clone(), organization_id);

        let service_name: NameVec<Test> = b"My Service".to_vec().try_into().unwrap();
        assert_ok!(PaymentChannels::create_service(
            RuntimeOrigin::signed(alice.clone()),
            organization.clone(),
            service_name.clone(),
            1,
            10,
            15,
            3,
            vec![].try_into().unwrap(),
        ));

        let service_id = PaymentChannels::hash_name(alice.clone(), service_name);
        let service = (organization, service_id.clone());

        assert_ok!(PaymentChannels::open_channel(
            RuntimeOrigin::signed(bob.clone()),
            service.clone(),
            100,
        ));


        let channel_id = PaymentChannels::hash_channel_id(bob.clone(), organization_id, service_id);
        let channel: ChannelSpecs<Test> = (bob.clone(), channel_id);
        let version = 1u32;
        let counter = 100u32;

        let (message, signature) = sign_message::<Test>(
            pair_bob.clone(),
            channel_id.clone(),
            version,
            counter.clone(),
        );

        assert_ok!(PaymentChannels::claim_channel_funds(
            RuntimeOrigin::signed(alice.clone()),
            channel.clone(),
            Some(counter),
            Some(signature.clone()),
        ));

        let mut wrapped_data: Vec<u8> = Vec::new();
        wrapped_data.extend(b"<Bytes>");
        wrapped_data.extend(&message);
        wrapped_data.extend(b"</Bytes>");

        let signature = MultiSignature::Sr25519(pair_bob.sign(&wrapped_data));
        assert_ok!(PaymentChannels::validate_signature(&message, &signature, &bob));
    })
}
