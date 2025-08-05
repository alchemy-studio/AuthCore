// use htycommons::{uuid};
// use htycommons::jwt::{jwt_decode_token, jwt_encode_token};
// use htycommons::web::HtyToken;
//
// #[ignored]
// #[test]
// fn test_jwt() {
//     let hty_id = uuid();
//     let msg = HtyToken {
//         token_id: uuid(),
//         hty_id: Some(hty_id.clone()),
//         app_id: None,
//         ts: current_local_datetime(),
//         roles: None,
//         tags: None
//     };
//     debug(format!("token: {:?}", msg).as_str());
//     match jwt_encode_token(msg) {
//         Ok(token) => {
//             debug(format!("encoded token: {:?}", token).as_str());
//             match jwt_decode_token(token) {
//                 Ok(decoded) => {
//                     //debug(format!("decoded token: {:?}", decoded.hty_id.unwrap()).as_str());
//                     assert_eq!(hty_id.clone(), decoded.hty_id.unwrap());
//                 }
//                 Err(e) => {
//                     panic!("{}", e.reason.unwrap())
//                 }
//             }
//         }
//         Err(e) => {
//             panic!("{}", e.reason.unwrap())
//         }
//     }
// }
//
// /*
//
// 2021-09-20 21:10:26,830 DEBUG [cn.alchemy.tas.com.Commons] (main) tokenJson -> {"id":"df4641a3-246c-49c7-83d9-02cfa5d4ba59","ts":"2021-09-20T21:10:26.794541"}
// 2021-09-20 21:10:26,914 DEBUG [cn.alchemy.tas.com.Commons] (main) JWT encoded -> eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ7XCJpZFwiOlwiZGY0NjQxYTMtMjQ2Yy00OWM3LTgzZDktMDJjZmE1ZDRiYTU5XCIsXCJ0c1wiOlwiMjAyMS0wOS0yMFQyMToxMDoyNi43OTQ1NDFcIn0iLCJpYXQiOjE2MzIxNDM0MjYsImV4cCI6MTYzMjE0MzcyNiwianRpIjoiMmRhZTVlNzAtNWQ1My00ZmFhLTg0NGUtMjQyZDJiZDg4NjVhIn0.kBGd9Zoz8j-ywUWB-1ck3DqVdODvVDWEM0TyMizGwtI
// 2021-09-20 21:10:26,925 DEBUG [cn.alchemy.tas.com.Commons] (main) JWT claim -> DefaultJWTCallerPrincipal{id='2dae5e70-5d53-4faa-844e-242d2bd8865a', name='{"id":"df4641a3-246c-49c7-83d9-02cfa5d4ba59","ts":"2021-09-20T21:10:26.794541"}', expiration=1632143726, notBefore=0, issuedAt=1632143426, issuer='null', audience=null, subject='{"id":"df4641a3-246c-49c7-83d9-02cfa5d4ba59","ts":"2021-09-20T21:10:26.794541"}', type='JWT', issuedFor='null', authTime=0, givenName='null', familyName='null', middleName='null', nickName='null', preferredUsername='null', email='null', emailVerified=null, allowedOrigins=null, updatedAt=0, acr='null', groups=[]}
//      */
// // todo: need structure adjustment
// #[test]
// #[ignore]
// fn test_java_interop() {
//     debug("---TEST JAVA INTEROP---");
//     let java_encoded = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ7XCJpZFwiOlwiZGY0NjQxYTMtMjQ2Yy00OWM3LTgzZDktMDJjZmE1ZDRiYTU5XCIsXCJ0c1wiOlwiMjAyMS0wOS0yMFQyMToxMDoyNi43OTQ1NDFcIn0iLCJpYXQiOjE2MzIxNDM0MjYsImV4cCI6MTYzMjE0MzcyNiwianRpIjoiMmRhZTVlNzAtNWQ1My00ZmFhLTg0NGUtMjQyZDJiZDg4NjVhIn0.kBGd9Zoz8j-ywUWB-1ck3DqVdODvVDWEM0TyMizGwtI";
//     let token = jwt_decode_token(java_encoded.to_string());
//     debug(format!("token -> {:?}", token).as_str());
//     assert_eq!("df4641a3-246c-49c7-83d9-02cfa5d4ba59".to_string(), token.unwrap().hty_id.unwrap());
// }
