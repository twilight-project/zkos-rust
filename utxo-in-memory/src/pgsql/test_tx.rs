pub fn deserialize_tx_string() -> transaction::transfer_tx::Transaction {
    bincode::deserialize(&hex::decode(return_transaction()).unwrap()).unwrap()
}
pub fn deserialize_tx_id() -> [u8; 32] {
    let tx_id: [u8; 32] =
        hex::decode("7DE9F3368FDBA3E23CED4AB9F425475C848CFAD5E62B692AE9DAB70B374F087F".to_string())
            .unwrap()
            .try_into()
            .unwrap();
    tx_id
}
pub fn tx_id_string() -> String {
    "7DE9F3368FDBA3E23CED4AB9F425475C848CFAD5E62B692AE9DAB70B374F087F".to_string()
}
pub fn return_transaction() -> String {
    "000000000000000001000000000000000000000000000000050500050000000000000000000000000000007c232d4f60090978087e792f871a6403bd8cf6ed650e6782e41065da3f545c6c019c973f03c3fcd531ffffed3025ba19382ddf97df31cb2c11d3b0888731cd9873606745b70cd44451020b62a0150fce13e1d8669a76d0c5f0c6d3951c022bf9468a000000000000003063373836613639373731666462323631333033666366373662386334626564396137656531353764306339333731333832383432663732353239376232356134643334323837613565643135663137316562373463663433303464303734376361653037356337353238346138336534653465313762306339643637303132353161633439386234310000000000000000007c232d4f60090978087e792f871a6403bd8cf6ed650e6782e41065da3f545c6c006c891f4cf5099d39f570c378e048b2faf645668d87eabfaf0727114f0a8e1a0c02e89c3fc5c4ac98c8f25a23aee72e5476cc591a1f8871fbe1cf17c53462b5568a00000000000000306335363739313637393836363161623339626666643763393239346338396234666230613964303463306562303434663435373438623161383062633038643736323266643761353536306362383761313636386663666263366133646237336263316462316637356534393031343635643963303533326534323866613831363165653939333963000000000000000000000000000000000000000000000000000000000000000000000000000000000000504966cac5226344ebb2b7417a9d5ce9d791f494dfbd89e895a7ee8e8e267002a8661006880d37d43650e910b564a24c478fd791ed7022e0584d20cd4a2fbb2e8a000000000000003063623235623238343665353161386131626339633531636365366133353935363137333531393461303832343261303866383039323961366161323132366532663134353766353736316330313630616535316561346461326638316534636636383939353764646162346439373439653264306137343239653939656534346636353334313433660000000000000000000000000000000000000000000000000000000000000000000000000000000000001e9ad1dbea6844c8949a75ec8548c202f2773505736bd4d7467d0b951f30a73ea0338b7d7f0cfcb920eabeeb005843f66a6c8153166cfc287d688f2faf88655c8a00000000000000306338303935383237666335356436646161303162623931616465356235326233373735356639396539333762316332333663303566643265343865613765623763613635306664303634356535613932636164643937333133303763326366396135653832333938633433623461643662333237326434336635333330353734633339633238336536000000000000000000000000000000000000000000000000000000000000000000000000000000000000504966cac5226344ebb2b7417a9d5ce9d791f494dfbd89e895a7ee8e8e267002a8661006880d37d43650e910b564a24c478fd791ed7022e0584d20cd4a2fbb2e8a000000000000003063623235623238343665353161386131626339633531636365366133353935363137333531393461303832343261303866383039323961366161323132366532663134353766353736316330313630616535316561346461326638316534636636383939353764646162346439373439653264306137343239653939656534346636353334313433660005000000000000000000000000000000a492834f937becb71b9bdabea23798346e6ade9f146767c1b3e0924c1698a656fa636cf094c1ca682977127b78b08ad1e3286b219f3cddc7394e31e3ae5da76c8a000000000000003063373836613639373731666462323631333033666366373662386334626564396137656531353764306339333731333832383432663732353239376232356134643334323837613565643135663137316562373463663433303464303734376361653037356337353238346138336534653465313762306339643637303132353161633439386234310000000000000000dc9b1c8e560ccf4dfe93f10125451703db0f9800422db19d3a2b21030c08ef14905fef92db41874b40965c45c3d38799ef27b3a627d919b4e82799a0b81f8b568a00000000000000306335363739313637393836363161623339626666643763393239346338396234666230613964303463306562303434663435373438623161383062633038643736323266643761353536306362383761313636386663666263366133646237336263316462316637356534393031343635643963303533326534323866613831363165653939333963000000000000000066410cbfe86de347ffbc7d2f3950ce10c5006c09f83e0704c07837b1e5a3fa75128030bcfb60ee3bc500712f4629fa7c149afac111424f21ab14b90de15f15388a000000000000003063623235623238343665353161386131626339633531636365366133353935363137333531393461303832343261303866383039323961366161323132366532663134353766353736316330313630616535316561346461326638316534636636383939353764646162346439373439653264306137343239653939656534346636353334313433660000000000000000aaaf141bab2a0ab6a910c655e19395769aff7a7cf4eeb7ab731c12dd8d81951d44a60d1884c44040c0c1f49bf2ef558ab42235b3dc2c9a214cb4bdb6dea90b218a000000000000003063383039353832376663353564366461613031626239316164653562353262333737353566393965393337623163323336633035666432653438656137656237636136353066643036343565356139326361646439373331333037633263663961356538323339386334336234616436623332373264343366353333303537346333396332383365360000000000000000e0bc7dee8f50c9dbbe200d26016895eccfa57086e4d8d74da72ac8f09c48bd572879b9984cadc95632b11ae96d6e9c5d9141350f71ca043d2cf638c408980b398a000000000000003063623235623238343665353161386131626339633531636365366133353935363137333531393461303832343261303866383039323961366161323132366532663134353766353736316330313630616535316561346461326638316534636636383939353764646162346439373439653264306137343239653939656534346636353334313433660500000000000000786a69771fdb261303fcf76b8c4bed9a7ee157d0c9371382842f725297b25a4d34287a5ed15f171eb74cf4304d0747cae075c75284a83e4e4e17b0c9d6701251864d8f99ea5fbfc3b0b5ce5c2a1a5476bb879189dfce59016afc8f9e4efab137decdad71174ae920bd2221c1be991cab95845118f9c676f52726ac2a55f00636567916798661ab39bffd7c9294c89b4fb0a9d04c0eb044f45748b1a80bc08d7622fd7a5560cb87a1668fcfbc6a3db73bc1db1f75e4901465d9c0532e428fa816166f1edf9617d68c71931fc6d09db5dd9f3b3a90f27ede715466997b5a8e682f4aeaf0ff42431b9f2c31b155d19adbc439e89c3c81283a0cfd00120a8327e466b25b2846e51a8a1bc9c51cce6a359561735194a08242a08f80929a6aa2126e2f1457f5761c0160ae51ea4da2f81e4cf689957ddab4d9749e2d0a7429e99ee44ff0d82ebf7574b1f5a5c4510455c600203d877ca86ac8c2e096771b05533e655c70040b541f87b2831c1a578343f24fe20d2be4d86b74f54d3917d2647e34d0458095827fc55d6daa01bb91ade5b52b37755f99e937b1c236c05fd2e48ea7eb7ca650fd0645e5a92cadd9731307c2cf9a5e82398c43b4ad6b3272d43f5330574cdc11fb3515aa4d2eb42603fcbc648dbd04bf3eaf3abf2120c36677020fbc7d25de46398da4ce5b87e9093bbc522a2151c716475019245692a042971786aa924bb25b2846e51a8a1bc9c51cce6a359561735194a08242a08f80929a6aa2126e2f1457f5761c0160ae51ea4da2f81e4cf689957ddab4d9749e2d0a7429e99ee44f4210a887f1f3cdab6d367e29b4b863e2bc3a86195037cea3eccbff5fedadab1d06e0e8840faf1f3430b22d63a0c2ea66ce660085b2c62dec548bc2e0997e14580500000000000000e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d768c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f340488711340c5584b0919acf5b8e9ec3c441fccc840f948440a428f1a65395e3d148d8f744a8cbca6a1afb7d052362438f7c8fba70be91a2274cc60ca327db2b0489eeb777e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d768c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f34048871134823294adf27597d17ffc0bc7d405504f1aeb1ea5ba050582dbf959cd46bedb740e4be1c52ec685565a79a7f88c4ba596816b34e36c5546a134db3847cc3d7134e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d768c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f3404887113420323e1c2893c4378184b0a7a190d230f2789600a41b376df09d194a467fb01d3239c18c0c92e408439bcf54cc5b3acfe94b4204dac35570799b8acbafc1b751e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d768c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f3404887113410db99020bdd3cdf958be48d3c48e797e2e8592c26735f812fc21ccaa78f33094e9855ec7045bd7e161ae7e48001a262c5c5b5b1fb471eb6d4141c5c1d83152be2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d768c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f340488711347ca163c78c1cb64c8d9cb162d737776139976fc4c3fdf7f67ef1adce942ac91186edb8107ff119d79a10ba44fea2b6fbdc7b8a5715891f64165512cea9bbe412010000000500000000000000b542fcf626f39b85af8feaf65cdab428ed79ef809c4b7570c3665b8a3afae40d2d2bf0d64ed11966c5c45c83a496a552af7793a183d62f160eec668f7287c705d997b2c119586df7d0a31c4f1d33ae9d465653d7f2deba84f585075167c62f0041f0f9d0666d968c4cb8a40e7f35d7915e7c784162a18bfd3dbb239267fe3c00d869539db65e940d8e35b223b8d5b82cfbbd1fe66a1d15bf6737e35bc6e4570b0500000000000000fdfdef7da06b6f2e23419887ab95bd09ae742e717f0e940d282d233fd7f620099f79c31500f06dfe573193fb34693fa174b76fb79416eab6575766f20451b90235530e36cdc7fa651a100bb3f4c227e2d6c5a1eb487ce4c403542c39919fe70c8574410fedbeea82d10e0ff609b573befee0e519bb2423fe4c74459163b50d0ebf9fc95b8fc1eb8016464c4b059ce5e9e419021f7634d5c4e504d28e5c936c05050000000000000026e486a76e41a383189b39a1e71874e9e04936a059402bb75239dcbebfccb00280467bdd6592ff4b67d2aa051d6d89d5bc8ce7749b5f0319b25d5cce443b41006c397bab1281d2fddac0d0e03371647783dd337d50cecef7f5950382256ba606afb53ba60aed1d57f8bfd2984acc165017f43183ae5d5fb35ce78ee7abdbd00a1cd4fa8f66dad3f605a7adcb1a97bf79795248d0e96050274db0d094bfffff030497742898992e4d17bb6510ee64c19c16c2289d0f6a096707870626ded866070200000000000000e2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d768c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f34048871134caeb4048d256aac423b031eb02ec50fd5169b5e6a592a2c66ac83fb9bfb2c869debf2df411ff7bcac503ff0d5d3957210572df712c19d3ebef85146c23a44d2de2f2ae0a6abc4e71a884a961c500515f58e30b6aa582dd8db6a65945e08d2d768c9240b456a9e6dc65c377a1048d745f94a08cdb7f44cbcd7b46f34048871134626c6672b75d5aa23846846c965f00516c89059e7093f0136ef33a62db51595b609c99c262c0ed257b40b9d6d274da2ba919a138f04c6a6147c5e5e09c75c33c010000000200000000000000dfe397e1063111e242d810f7c6c81b9990c362b533c0bc288a99a1794de99006cbedfd037134f4b3214492bdf11b5382fb2beef7c053db5fe3ecb6e324e411040200000000000000da64aeeeea63fcd5375935dde8491c3fd7c019a5702b2089c5922f2e18b1890549262d9c35aadd12799f737bfd9487013c09ed08fc370b12869eaa888c693209020000000000000098263af6c5231390598336dc4b7514ded828e04721f870567d1a8417a42c05001380918c9f061aa506a9ccd9dbd5d78a2bedfff6862bc44b2900697982361006b8db5e6347f2e4f3a9ddde736ffd750a31062c4a0e693f418508ea04690ab80d0500000000000000a002000000000000e8ee3f9fcaef0ba22bf664664fc85082203586c0e7c66859b70b95b732cd3d6522e5053efa79388d8ec904cb729fc2c47f7a43fd3dd6e1fbbf7c950e3248ee6214b2ae13599f22ec4bed00d66261cd9143d39005f7f62c558be147e9aa4f6f79f64d0e7874af3d5dc284105333d3d1be118ba95ed2ae99f39b46996ee3c77a43c34ced72b036f65e0ef31fb8a880ae175799434168e13bb85ade1d989fa4ba0131630af7acd304b6ea04ac92fdd3e257032dbea48f63f7b8fb9ee4b820863c0189394d9de7a8384233b937e12e054b0d93b0c1c68eddf44848468877e3810702a2b98d66af77c3673a41127fd53448369dc555e491fc9a7c11ed1dc7e15f120c0285becfcaafba45a299f71768f1a2d3f6fb5885c65b960d40fdcbda41813245c856f0d44e259ec04ebdc315cc780cbcb98bb5afa9b8a8e76a4084137119ac1db6af8ce211e45dfb5549f72aaf11ebe9a4afcecc442d345d85f1869c2a984b76d888ac7a1617bd0dc794c375fe8cb29869572a577d8100b2a65245e3a61544342c08cd2c5eda0bae1ec7dce54801eb16fabed717adfcc82f32cb8eb7be49a16e348e522237fed7341430deb394e815b2d9819e472bb6a3e4f48c6ae62441487626a4e0bb264fbb865bf2c6e60c6bf02af2efb6e55671fe9ef7e900f2b1c9f37b9c384e15f4cfa664748a20dcecb3863b1242b002d986c5ee0fa263dd2868052fe0d9723913c68cbad4467655abf12803abf0d28d6c03119f5a77f09d9365bc32c89aa2a3552a3b1db3e4c189278457217bf55bf6b579b418b69475b5f0bf1a35188e6bf46535b3861ec3361e82a673a60baeea14043e471497d94a4361c26273a8ff4efc7d875ba50862275391229cfc0e419295a4c182458b5ff9986e54390489ee3899186a57c39f4cdc9d5535db8349878877022c1547d4557aa94c29700ea002000000000000aeee3109f89dfb1848ef74f35c266829a02548782d88688299a25b0e4eee160d8276846bd7d015004161d001e8ddd749b6a35e0a26aac66a0e7adb107e93d946c4fd4b35386d539a6d602e5b81effa112dd02e61078090900f2973b7f617c04be0facc37a9e0c0472a71a74e989170cf61b1878137f7e5b74e0336e9861fed044b77972ad90443278dcba72917387d95a8ca6f42467fdbddf8fb6495ae533e04a78c3c676894b9fb44bf916b50e9475476f564fc9678cc468d417b7adbad1f0e3334f7fef3889fac3a06e6018ea75a54adfbc851ec6fac3606b6bdd3dea7c2007ce14a85d6628e373974930839093cd8fdbf73b4b43b17d4163b54acd4c3ec21fc141028a985d82e8d8ad2638f98f96ffb56a34013b21db4534de7a885b57c66000ad6825727a4cda09556063ac75885c2918222348b96fa16e4f40ce30c2840f097c98e525930fc18783c0080e76b762ac55356d9d9f0b8c3aef086104b62217a7af1ffc282da015e1c5e22664cd2d182e8cf97305727833fef918bb07ca11e0ebfbd8cbd96306894b7106ed3b89428b92762f79d7e0725295b15d192879a474ede198c711973f304857bd246d15539f99d1ebfaa0cc5d10f150e6c86d6a000c6cf4e8f464538644ce4f38fd332c404753d58509cc61185a41058d46745b43b828ce105287c39c2d1044662d37132d8db74624b0222dc802c07b1a34716193906a9a13b93bfc2b534c5c0936f52a6e02fa83804ec287878b5bd915b18146d200a5cb1e6fb9419d9d8efa31c62cfd05d5c351cc0da733d8c095f9ea78658bf0e3ef7a8f5aaa7b33b63234ff1b5bc52384d3cc166f30bb3b7a2e142e29a9cf45580f97c7021f54ccff18e3c870d0da6cc996a088e17bc7a0e5038bec6cd72500df8db0566bb1f1bb10d901b05b66554650aa6e58d51b8c9a45037879c644a340fa002000000000000e0a8b67b540f63e9fa150afbd175d5f557e7e5d92bd8a8927d9fd1d6513d6d5eca16f9b874c11f7a25573dfdf5e3f84b08e147c5908ee093c8a4331fb6126b5a3e53a27c758934d6f5e25a3ab72da1515222f099d41ba995ff4a3ff1b1e7de432c7ad0cc0ba9d8596d38cd8593c11d513630dba179aec275bcefe0fc7f9cf01842f094e9327cca1cb00e9320c4aea554f890fc0b298dee458d5aaa7c8c481b089f55a4951aa5019fbe74a699608ab4873a4e8a33ac062853f62effdbebf6a20dccfac1149ab7076a59bada9d76173f387ce378d2778e63b6cb4708392030ff028805ec32c24004ee5ee0d0a356f873fad9b8b2f729b6d1d9f937add6ae019d798cdd581baccb6265af29861a095ed5da98f7710d569d7c0926bd5097334eee63a4fed041d5bb2ed014ee130606f96495c5ca08b151784f6ac8c1d346b7808a5ba42532b32a6829d5fdd6abfe7a1a22641cc451202f7605d00e338590bdf6c73f46e16b157817cf671a588647240f353c171eaa163b5e1b4b53fc5ceb97aa336756ea5bb05b2e45374a3fca633453ae22e81b54df2bec0bc2a7a37b64079bab6c8e2cefc27570e9580b34c1dd679cdc696f1fd0ed75262047bfd5495c2a07bc7456be7a39949244573c9c9e2fdeb4e84dd3f52d51daa009e1dee326d2698ede7aec3be7d8ba275e1c29626cba2db06a2ff38fe89021ac41cd59c6c18526466644140e0216b24a07330080f6b05826ea5626808a47515d0cbd46403c9cd0984b27761a11c5ac10719e67c971f21015048e3a8a063cf01ef4c69aa3be96856d407fe2e020315c248910f739d737427d7f4a9d891a2bccbed168862ec2779f8257059031899ee862280f329814a3e8d4a3ae50f9fba8a2299c47f519b2cbb098640acac3c987f5f7231090d284fb61bfeeca75ddd123bfdc03d0379fe0ae9d7dea09a00200000000000074c96c4f2aef7d91d8e9d57d4a58b9b2afcebed56a1c454f67128f65e828e24074df47750b35705b5251019dd42ceea30b755e86a85b929285a570644a549f177a09d8764af1d06ee39f679ad2c91714c815964cfcd5bb9f0772e46dc01fd670be2888b638738fb547995d6657703b5ab22eb01dde2f097422cec94c83aeea12b1b7abdef6b7032ca91bb8db5eb880dcbce320b6a4f4348e09c114f10fbe9801911e7872f7fd70dc4b585732f2334a4d865ff2def5bb3a208df815f27e12bf05964d2f2a8bd549341af15556e7f4fcccf2f3746a748430b1ed1ead012973e2056291b6e61a7e4f7dbe1d7bb5d35ad1da57c31bb94e081500ce1feac13278e32762b3a752e70ffcc94dc172af77b01fc92038908f869ece056b42388da4ba9928581d2845f78ae989f2f0012193dfcfe821402471c85b5b6576f47a0a65e78a3900738ffef0a5ccff7204caafa9676137e96d678003153de8398d046237d29411dc411589f889e3cc4c0d4218bef7c1904f3e7bf0921152d9e3cf0bf290d7375d5e9f3ac49c6e385be1226667f8f45655484b374c0f81d581324230d04d7d0f624cbe01108b5ea472cbf09536a52667b5a1e8833d9ed08bc24d1f6e19042c573db23bc31a6e781b03282a4a3ce5e561d0017fc31659dc84ab848dd1021f2ccb22de6d33cebf696e5d3e58b3cab92a0f241ca5d76cb0c6d19a80eddfe0d52b2d50c8b3d0ef6a353af6c6ca101867518ce01632ccd1beb0d9eb78a07cf37cfdb50bac3233f7758ecf6aced6877570bea5992e42b5dc0b8c6b721b8713d82795c8372a4b2b22f5eaeb9a65f98356690f918160e345e851734228ea69b2b49f793b67456a037d13f27088ce93f210cf0108bf866583c5e7cf8b1276b0d2ff2afbc0064f907b79d0068c18ae2cf16a75c88a6994e80e77d53cf07d0405e3171d6a8503a002000000000000defe119a5bb0089e430fd2713d80e11fa973066deffe72b46a53a4825a2ce66f4a8821ca0848f9316e64902e42d5d6c52b061b824fc88482035b9cd852051b490add5d7172068a397f95562a3a3591f23adf64baaa34a5b4cecb45271cb90664da31a7234b335df50cf24108683179a4c4d111f8e2c4eb50c08bc18fed9b602c4cb17f6901f20ca30b364470d331c0fb6c3ba532d8a5c0476ec5140adccd580533522d4470798e0f9972803c55a7d0e6bcd37ea1706c5f51e8e6fbe1acf592092be9a440cff2d7b466cac04daa80cc1da686af861a4b3abf51c548378b7c31097e14e8a8734dcd1dcea6494da7a6c81e69ed206c140dfeedc1db8532bf501f6f10f13f96af777f8a7981b554a970d6697189e8e8f253a5bd3f314165a92ab95b1c23e701c72841f86894b4e9ce2793165a6c6e3899798011c964913dbd692c11181520a74f4c4b24b6a785fb7de34fcc365af706332fb5db939db8f8afa0fe2294a9b2018fcb4489933e7fe4482ff8be5bedd6dafb282eb1f04956eef70d183680c845b3ef4f1838fe4033e3d8677097006ca02d75c7e38eb503f195626ba7052c389f7ea97d5f7b3333245854f9e715e4053212103f9b44b7e1be8d641a89600e116d354744b40750b7163c12e5aca69616ee33fb1cfae1ab96e470a826ab576a7dd9d399a726a9e8cf39e20b81c01381044dc23ff0ef061127de275d4dd146aa7e7e526f102d7882ac93e70f3c59be1abb67597beddf57bfddb926c0b043754eb5df8b60123179e33d3bf48e8877c62f473a97dfc85d7bc18e14dcba54c27c6ec681f98c76c06cb58d1a436ce69158c8cb190dd6e3ecc93d3b452c26f1e924065e318024661be166cbf7c3bddd8461a007c16fb42db0f8091be23850a4d90d180ac2b500663262356ee76e59b28d197e516679fc14e3c5fb7b4613f5a2040103000000000000000000".to_string()
}
