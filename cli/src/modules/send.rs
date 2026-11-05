use crate::imports::*;

#[derive(Default, Handler)]
#[help("Send a Celenium transaction to a public address")]
pub struct Send;

impl Send {
    async fn main(self: Arc<Self>, ctx: &Arc<dyn Context>, argv: Vec<String>, _cmd: &str) -> Result<()> {
        // address, amount, priority fee
        let ctx = ctx.clone().downcast_arc::<CeleniumCli>()?;

        let account = ctx.wallet().account()?;

        if argv.len() < 2 {
            tprintln!(ctx, "usage: send <address> <amount> <priority fee>");
            return Ok(());
        }

        let address = Address::try_from(argv.first().unwrap().as_str())?;
        let amount_pradha = try_parse_required_nonzero_celenium_as_pradha_u64(argv.get(1))?;
        // TODO fee_rate
        let fee_rate = None;
        let priority_fee_pradha = try_parse_optional_celenium_as_pradha_i64(argv.get(2))?.unwrap_or(0);
        let outputs = PaymentOutputs::from((address.clone(), amount_pradha));
        let abortable = Abortable::default();
        let (wallet_secret, payment_secret) = ctx.ask_wallet_secret(Some(&account)).await?;

        // let ctx_ = ctx.clone();
        let (summary, _ids) = account
            .send(
                outputs.into(),
                fee_rate,
                priority_fee_pradha.into(),
                None,
                wallet_secret,
                payment_secret,
                &abortable,
                Some(Arc::new(move |_ptx| {
                    // tprintln!(ctx_, "Sending transaction: {}", ptx.id());
                })),
            )
            .await?;

        tprintln!(ctx, "Send - {summary}");
        tprintln!(ctx, "\nSending {} CEN to {address}, tx ids:", pradha_to_celenium_string(amount_pradha));
        // tprintln!(ctx, "{}\n", ids.into_iter().map(|a| a.to_string()).collect::<Vec<_>>().join("\n"));

        Ok(())
    }
}
