use decimal::*;
use traceable_result::*;
use alloc::string::ToString;
use crate::math::consts::*;
use crate::math::types::{fixed_point::FixedPoint, token_amount::TokenAmount};

#[decimal(24)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo)
)]
pub struct SqrtPrice(pub u128);

impl SqrtPrice {
    pub fn from_tick(i: i32) -> TrackableResult<Self> {
        calculate_sqrt_price(i)
    }

    pub fn big_div_values_to_token(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<TokenAmount> {
        let nominator = u256_to_u320(nominator);
        let denominator = u256_to_u320(denominator);

        let intermediate_u320 = nominator
            .checked_mul(Self::one::<U320>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = checked_u320_to_u256(intermediate_u320)
            .ok_or_else(|| err!("Can't parse from u320 to u256"))?
            .checked_div(Self::one::<U256>())
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;
        Ok(TokenAmount(result))
    }

    pub fn big_div_values_to_token_up(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<TokenAmount> {
        let nominator = u256_to_u320(nominator);
        let denominator = u256_to_u320(denominator);

        let intermediate_u320 = nominator
            .checked_mul(Self::one::<U320>())
            .ok_or_else(|| err!(TrackableError::MUL))?
            .checked_add(denominator - 1)
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(denominator)
            .ok_or_else(|| err!(TrackableError::DIV))?;

        let result = checked_u320_to_u256(intermediate_u320)
            .ok_or_else(|| err!("Can't parse from u320 to u256"))?
            .checked_add(Self::almost_one::<U256>())
            .ok_or_else(|| err!(TrackableError::ADD))?
            .checked_div(Self::one::<U256>())
            .ok_or_else(|| err!(TrackableError::DIV))?
            .try_into()
            .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?;
        Ok(TokenAmount::new(result))
    }

    pub fn big_div_values_up(nominator: U256, denominator: U256) -> SqrtPrice {
        SqrtPrice::new({
            nominator
                .checked_mul(Self::one::<U256>())
                .unwrap()
                .checked_add(denominator.checked_sub(U256::from(1u32)).unwrap())
                .unwrap()
                .checked_div(denominator)
                .unwrap()
                .try_into()
                .unwrap()
        })
    }

    pub fn checked_big_div_values(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<SqrtPrice> {
        Ok(SqrtPrice::new(
            nominator
                .checked_mul(Self::one::<U256>())
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_div(denominator)
                .ok_or_else(|| err!(TrackableError::DIV))?
                .try_into()
                .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
    }

    pub fn checked_big_div_values_up(
        nominator: U256,
        denominator: U256,
    ) -> TrackableResult<SqrtPrice> {
        let denominator = u256_to_u320(denominator);

        Ok(SqrtPrice::new(
            u256_to_u320(nominator)
                .checked_mul(Self::one::<U320>())
                .ok_or_else(|| err!(TrackableError::MUL))?
                .checked_add(
                    denominator
                        .checked_sub(U320::from(1u32))
                        .ok_or_else(|| err!(TrackableError::SUB))?,
                )
                .ok_or_else(|| err!(TrackableError::ADD))?
                .checked_div(denominator)
                .ok_or_else(|| err!(TrackableError::DIV))?
                .try_into()
                .map_err(|_| err!(TrackableError::cast::<Self>().as_str()))?,
        ))
    }
}

pub fn check_tick_to_sqrt_price_relationship(
    tick_index: i32,
    tick_spacing: u16,
    sqrt_price: SqrtPrice,
) -> TrackableResult<bool> {
    if tick_index + tick_spacing as i32 > MAX_TICK {
        let max_tick = get_max_tick(tick_spacing);
        let max_sqrt_price = ok_or_mark_trace!(SqrtPrice::from_tick(max_tick))?;
        if sqrt_price != max_sqrt_price {
            return Ok(false);
        }
    } else {
        let lower_bound = ok_or_mark_trace!(SqrtPrice::from_tick(tick_index))?;
        let upper_bound =
            ok_or_mark_trace!(SqrtPrice::from_tick(tick_index + tick_spacing as i32))?;
        if sqrt_price >= upper_bound || sqrt_price < lower_bound {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn calculate_sqrt_price(tick_index: i32) -> TrackableResult<SqrtPrice> {
    // checking if tick be converted to sqrt_price (overflows if more)
    let tick = tick_index.abs();

    if tick > MAX_TICK {
        return Err(err!("tick over bounds"));
    }

    let mut sqrt_price = FixedPoint::from_integer(1);

    if tick & 0x1 != 0 {
        sqrt_price *= FixedPoint::new(1000049998750);
    }
    if tick & 0x2 != 0 {
        sqrt_price *= FixedPoint::new(1000100000000);
    }
    if tick & 0x4 != 0 {
        sqrt_price *= FixedPoint::new(1000200010000);
    }
    if tick & 0x8 != 0 {
        sqrt_price *= FixedPoint::new(1000400060004);
    }
    if tick & 0x10 != 0 {
        sqrt_price *= FixedPoint::new(1000800280056);
    }
    if tick & 0x20 != 0 {
        sqrt_price *= FixedPoint::new(1001601200560);
    }
    if tick & 0x40 != 0 {
        sqrt_price *= FixedPoint::new(1003204964963);
    }
    if tick & 0x80 != 0 {
        sqrt_price *= FixedPoint::new(1006420201726);
    }
    if tick & 0x100 != 0 {
        sqrt_price *= FixedPoint::new(1012881622442);
    }
    if tick & 0x200 != 0 {
        sqrt_price *= FixedPoint::new(1025929181080);
    }
    if tick & 0x400 != 0 {
        sqrt_price *= FixedPoint::new(1052530684591);
    }
    if tick & 0x800 != 0 {
        sqrt_price *= FixedPoint::new(1107820842005);
    }
    if tick & 0x1000 != 0 {
        sqrt_price *= FixedPoint::new(1227267017980);
    }
    if tick & 0x2000 != 0 {
        sqrt_price *= FixedPoint::new(1506184333421);
    }
    if tick & 0x4000 != 0 {
        sqrt_price *= FixedPoint::new(2268591246242);
    }
    if tick & 0x8000 != 0 {
        sqrt_price *= FixedPoint::new(5146506242525);
    }
    if tick & 0x0001_0000 != 0 {
        sqrt_price *= FixedPoint::new(26486526504348);
    }
    if tick & 0x0002_0000 != 0 {
        sqrt_price *= FixedPoint::new(701536086265529);
    }

    // Parsing to the Sqrt_price type by the end by convention (should always have 12 zeros at the end)
    Ok(if tick_index >= 0 {
        SqrtPrice::checked_from_decimal(sqrt_price)
            .map_err(|_| err!("calculate_sqrt_price: parsing from scale failed"))?
    } else {
        SqrtPrice::checked_from_decimal(
            FixedPoint::from_integer(1)
                .checked_div(sqrt_price)
                .map_err(|_| err!("calcaule_sqrt_price::checked_div division failed"))?,
        )
        .map_err(|_| err!("calculate_sqrt_price: parsing scale failed"))?
    })
}

pub fn get_max_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MAX_TICK / tick_spacing * tick_spacing
}

pub fn get_min_tick(tick_spacing: u16) -> i32 {
    let tick_spacing = tick_spacing as i32;
    MIN_TICK / tick_spacing * tick_spacing
}

pub fn get_max_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let max_tick = get_max_tick(tick_spacing);
    SqrtPrice::from_tick(max_tick).unwrap()
}

pub fn get_min_sqrt_price(tick_spacing: u16) -> SqrtPrice {
    let min_tick = get_min_tick(tick_spacing);
    SqrtPrice::from_tick(min_tick).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sqrt_price() {
        {
            let sqrt_price = SqrtPrice::from_tick(20_000).unwrap();
            // expected 2.718145925979
            // real     2.718145926825...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(2718145925979u128, 12));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(200_000).unwrap();
            // expected 22015.455979766288
            // real     22015.456048527954...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(22015455979766288u128, 12));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(-20_000).unwrap();
            // expected 0.367897834491
            // real     0.36789783437712...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(367897834491u128, 12));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(-200_000).unwrap();
            // expected 0.000045422634
            // real     0.00004542263388...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(45422634u128, 12))
        }
        {
            let sqrt_price = SqrtPrice::from_tick(0).unwrap();
            assert_eq!(sqrt_price, SqrtPrice::from_integer(1));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
            // expected 65535.383934512647
            // real     65535.384161610681...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(65535383934512647u128, 12));
            assert_eq!(sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE));
        }
        {
            let sqrt_price = SqrtPrice::from_tick(MIN_TICK).unwrap();
            // expected 0.000015258932
            // real     0.0000152589324...
            assert_eq!(sqrt_price, SqrtPrice::from_scale(15258932u128, 12));
            assert_eq!(sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE));
        }
    }

    #[test]
    fn test_domain_calculate_sqrt_price() {
        // over max tick
        {
            let tick_out_of_range = MAX_TICK + 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
        // below min tick
        {
            let tick_out_of_range = -MAX_TICK - 1;
            let (_, cause, stack) = SqrtPrice::from_tick(tick_out_of_range).unwrap_err().get();
            assert_eq!("tick over bounds", cause);
            assert_eq!(1, stack.len());
        }
    }

    #[test]
    fn test_sqrt_price_limitation() {
        {
            let global_max_sqrt_price = SqrtPrice::from_tick(MAX_TICK).unwrap();
            assert_eq!(global_max_sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE)); // ceil(log2(this)) = 96
            let global_min_sqrt_price = SqrtPrice::from_tick(-MAX_TICK).unwrap();
            assert_eq!(global_min_sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE)); // floor(log2(this)) = 63
        }
        {
            let max_sqrt_price = get_max_sqrt_price(1);
            let max_tick: i32 = get_max_tick(1);
            assert_eq!(max_sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(MAX_SQRT_PRICE)
            );

            let max_sqrt_price = get_max_sqrt_price(2);
            let max_tick: i32 = get_max_tick(2);
            assert_eq!(max_sqrt_price, SqrtPrice::new(MAX_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(MAX_SQRT_PRICE)
            );

            let max_sqrt_price = get_max_sqrt_price(5);
            let max_tick: i32 = get_max_tick(5);
            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(65525554855399275000000000000)
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(65525554855399275000000000000)
            );

            let max_sqrt_price = get_max_sqrt_price(10);
            let max_tick: i32 = get_max_tick(10);
            assert_eq!(max_tick, 221810);
            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(65509176333123237000000000000)
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(65509176333123237000000000000)
            );

            let max_sqrt_price = get_max_sqrt_price(100);
            let max_tick: i32 = get_max_tick(100);
            assert_eq!(max_tick, 221800);

            assert_eq!(
                max_sqrt_price,
                SqrtPrice::new(65476431569071896000000000000)
            );
            assert_eq!(
                SqrtPrice::from_tick(max_tick).unwrap(),
                SqrtPrice::new(65476431569071896000000000000)
            );
        }
        {
            let min_sqrt_price = get_min_sqrt_price(1);
            let min_tick: i32 = get_min_tick(1);
            assert_eq!(min_sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(MIN_SQRT_PRICE)
            );

            let min_sqrt_price = get_min_sqrt_price(2);
            let min_tick: i32 = get_min_tick(2);
            assert_eq!(min_sqrt_price, SqrtPrice::new(MIN_SQRT_PRICE));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(MIN_SQRT_PRICE)
            );

            let min_sqrt_price = get_min_sqrt_price(5);
            let min_tick: i32 = get_min_tick(5);
            assert_eq!(min_sqrt_price, SqrtPrice::new(15261221000000000000));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(15261221000000000000)
            );

            let min_sqrt_price = get_min_sqrt_price(10);
            let min_tick: i32 = get_min_tick(10);
            assert_eq!(min_tick, -221810);
            assert_eq!(min_sqrt_price, SqrtPrice::new(15265036000000000000));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(15265036000000000000)
            );

            let min_sqrt_price = get_min_sqrt_price(100);
            let min_tick: i32 = get_min_tick(100);
            assert_eq!(min_tick, -221800);
            assert_eq!(min_sqrt_price, SqrtPrice::new(15272671000000000000));
            assert_eq!(
                SqrtPrice::from_tick(min_tick).unwrap(),
                SqrtPrice::new(15272671000000000000)
            );
        }
    }
}
