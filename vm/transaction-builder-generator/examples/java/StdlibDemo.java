// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

import java.util.Arrays;
import java.util.ArrayList;

import com.facebook.serde.Bytes;
import com.facebook.serde.Serializer;
import com.facebook.serde.Unsigned; // used as documentation.
import com.facebook.lcs.LcsSerializer;
import org.starcoin.stdlib.Stdlib;
import org.starcoin.types.AccountAddress;
import org.starcoin.types.Identifier;
import org.starcoin.types.Script;
import org.starcoin.types.StructTag;
import org.starcoin.types.TypeTag;

public class StdlibDemo {

    static AccountAddress make_address(byte[] values) {
        assert values.length == 16;
        Byte[] address = new Byte[16];
        for (int i = 0; i < 16; i++) {
            address[i] = Byte.valueOf(values[i]);
        }
        return new AccountAddress(address);
    }

    public static void main(String[] args) throws Exception {
        StructTag.Builder builder = new StructTag.Builder();
        builder.address = make_address(new byte[]{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1});
        builder.module = new Identifier("LBR");
        builder.name = new Identifier("LBR");
        builder.type_params = new ArrayList<org.starcoin.types.TypeTag>();
        StructTag tag = builder.build();

        TypeTag token = new TypeTag.Struct(tag);

        AccountAddress payee = make_address(
            new byte[]{0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22});

        @Unsigned Long amount = Long.valueOf(1234567);
        Script script =
            Stdlib.encode_peer_to_peer_with_metadata_script(token, payee, new Bytes(new byte[]{}),java.math.BigInteger.valueOf(amount), new Bytes(new byte[]{}));

        Serializer serializer = new LcsSerializer();
        script.serialize(serializer);
        byte[] output = serializer.get_bytes();

        for (byte o : output) {
            System.out.print(((int) o & 0xFF) + " ");
        };
        System.out.println();
    }

}
