use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct Data {
    pub service: u16,
    pub data: String,
}

impl Data {
    pub async fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let len = self.data.len() as u32;
        buf.write_u32(len).await.unwrap();
        buf.write_u16(self.service).await.unwrap();
        buf.extend_from_slice(&self.data.to_string().as_bytes());
        buf
    }
}