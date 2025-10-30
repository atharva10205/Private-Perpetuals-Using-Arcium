use arcis_imports::*;

#[encrypted]
mod circuits {
    use arcis_imports::*;

    #[derive(Clone, Copy)]
    pub struct EncryptedTradeParams {
        pub leverage: u8,
        pub margin: f64,
        pub size: f64,
        pub entry_price: f64,
        pub side_flag: u8,
        pub pair_id: u8,
    }

    #[derive(Clone, Copy)]
    pub struct EncryptedComputeResponse {
        pub ok: u8,
        pub msg: [u8; 64],
        pub validated_size: f64,
        pub validated_price: f64,
    }

   
    fn to_fixed_bytes(input: &[u8]) -> [u8; 64] {
        let mut buf = [0u8; 64];
        if input.len() > 0 { buf[0] = input[0]; }
        if input.len() > 1 { buf[1] = input[1]; }
        if input.len() > 2 { buf[2] = input[2]; }
        if input.len() > 3 { buf[3] = input[3]; }
        if input.len() > 4 { buf[4] = input[4]; }
        if input.len() > 5 { buf[5] = input[5]; }
        if input.len() > 6 { buf[6] = input[6]; }
        if input.len() > 7 { buf[7] = input[7]; }
        if input.len() > 8 { buf[8] = input[8]; }
        if input.len() > 9 { buf[9] = input[9]; }
        if input.len() > 10 { buf[10] = input[10]; }
        if input.len() > 11 { buf[11] = input[11]; }
        if input.len() > 12 { buf[12] = input[12]; }
        if input.len() > 13 { buf[13] = input[13]; }
        if input.len() > 14 { buf[14] = input[14]; }
        if input.len() > 15 { buf[15] = input[15]; }
        buf
    }

    #[instruction]
    pub fn open_position_v1(
        input_ctxt: Enc<Shared, EncryptedTradeParams>,
    ) -> Enc<Shared, EncryptedComputeResponse> {
        let params = input_ctxt.to_arcis();

        // Default invalid response
        let mut resp = EncryptedComputeResponse {
            ok: 0,
            msg: to_fixed_bytes(b"validation failed"),
            validated_size: 0.0,
            validated_price: 0.0,
        };

        if params.leverage < 1 || params.leverage > 100 {
            resp.msg = to_fixed_bytes(b"invalid leverage");
        } else if params.margin < 0.001 {
            resp.msg = to_fixed_bytes(b"insufficient margin");
        } else if params.size <= 0.0 {
            resp.msg = to_fixed_bytes(b"invalid size");
        } else if params.entry_price <= 0.0 {
            resp.msg = to_fixed_bytes(b"invalid entry price");
        } else {
            resp.ok = 1;
            resp.msg = to_fixed_bytes(b"position validated successfully");
            resp.validated_size = params.size;
            resp.validated_price = params.entry_price;
        }

        input_ctxt.owner.from_arcis(resp)
    }

   
    #[instruction]
    pub fn close_position_v1(
        input_ctxt: Enc<Shared, EncryptedTradeParams>,
    ) -> Enc<Shared, EncryptedComputeResponse> {
        let params = input_ctxt.to_arcis();

        let resp = EncryptedComputeResponse {
            ok: 1,
            msg: to_fixed_bytes(b"position closed successfully"),
            validated_size: params.size,
            validated_price: params.entry_price,
        };

        input_ctxt.owner.from_arcis(resp)
    }

 
    #[instruction]
    pub fn adjust_collateral_v1(
        input_ctxt: Enc<Shared, EncryptedTradeParams>,
    ) -> Enc<Shared, EncryptedComputeResponse> {
        let params = input_ctxt.to_arcis();

        let resp = if params.margin < 0.001 {
            EncryptedComputeResponse {
                ok: 0,
                msg: to_fixed_bytes(b"margin too low"),
                validated_size: 0.0,
                validated_price: 0.0,
            }
        } else {
            EncryptedComputeResponse {
                ok: 1,
                msg: to_fixed_bytes(b"collateral adjusted"),
                validated_size: params.size,
                validated_price: params.entry_price,
            }
        };

        input_ctxt.owner.from_arcis(resp)
    }
}
