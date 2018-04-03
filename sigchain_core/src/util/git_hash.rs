use protocol::logs::{GitCommitSignature, GitSignatureResult};
use super::sha1;
use super::pgp::*;

pub trait GitHash {
    fn git_hash(&self) -> Option<[u8; 20]>;

    fn git_hash_hex_string(&self) -> Option<String> {
        let hash = self.git_hash();
        hash.map(|h| {
            let hex:Vec<String> = h.iter().map(|b| format!("{:02x}", b)).collect();
            hex.join("")
        })
    }

    fn git_hash_short_hex_string(&self) -> Option<String> {
        let hash = self.git_hash();
        hash.map(|h| {
            let hex:Vec<String> = h[0..4].iter().map(|b| format!("{:02x}", b)).collect();
            let mut hex_string = hex.join("");
            hex_string.pop(); // to get 7 chars

            hex_string
        })
    }
}

impl GitHash for GitCommitSignature {
    fn git_hash(&self) -> Option<[u8; 20]> {
        let signature = match self.result.clone() {
            GitSignatureResult::Signature(ref sig) => {
                match pgp_signature_ascii_armor_string(sig) {
                    Err(_) => { return None; }
                    Ok(armored_sig) => {
                        armored_sig.clone()
                    }
                }
            }
            _ => { return None; }
        };

        let mut commit_data: Vec<u8> = Vec::new();

        // tree
        commit_data.extend_from_slice(b"tree ");
        commit_data.extend_from_slice(self.tree.as_bytes());
        commit_data.extend_from_slice(b"\n");

        // parent
        for parent in self.parents.clone() {
            commit_data.extend_from_slice(b"parent ");
            commit_data.extend_from_slice(parent.as_bytes());
            commit_data.extend_from_slice(b"\n");
        }

        // author
        commit_data.extend_from_slice(b"author ");
        commit_data.extend_from_slice(self.author.as_bytes());
        commit_data.extend_from_slice(b"\n");

        // committer
        commit_data.extend_from_slice(b"committer ");
        commit_data.extend_from_slice(self.committer.as_bytes());
        commit_data.extend_from_slice(b"\n");

        // sig
        commit_data.extend_from_slice(b"gpgsig");
        let space_adjusted_signature: Vec<String> = signature.split("\n").map(|s| format!(" {}", s)).collect();
        commit_data.extend_from_slice(space_adjusted_signature.join("\n").as_bytes());
        commit_data.extend_from_slice(b"\n");

        //message
        commit_data.extend(self.message.clone());

        // precommit
        let mut full_commit_data: Vec<u8> = Vec::new();
        let commit_len_payload = format!("commit {}", commit_data.len());
        full_commit_data.extend_from_slice(commit_len_payload.as_bytes());
        full_commit_data.push(0x00);

        full_commit_data.extend(commit_data);

        let mut hash = sha1::Sha1::new();
        hash.update(&full_commit_data);
        return Some(hash.digest().bytes());
    }
}
