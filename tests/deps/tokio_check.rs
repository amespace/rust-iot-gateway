/// 测试tokio异步运行时
#[tokio::test]
async fn tokio_check() {
    let (tx ,  mut rx) = tokio::sync::mpsc::channel::<u32>(8);
    tx.send(42).await.unwrap();
    assert_eq!(rx.recv().await, Some(42));

    let jh = tokio::spawn(async {7});
    assert_eq!(jh.await.unwrap(), 7);

}