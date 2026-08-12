use crate::dicts::PronContext;

#[must_use]
pub fn english_word_to_pyogi(ctx: &PronContext, word: &[u8]) -> Option<String> {
    ktts_kma::english::english_word_to_pyogi(&ctx.eng, word)
}

#[must_use]
pub fn english_prosess(word: &[u8]) -> String {
    ktts_kma::english::english_prosess(word)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prosess_rules_known_words() {
        let ctx = PronContext::empty();
        let cases = [
            ("are", "a"),
            ("card", "kad_"),
            ("wifi", "waipi"),
            ("xyz", "k_sij_"),
            ("azxq", "8j_k_s_k_"),
        ];
        for (word, expect) in cases {
            let got = english_word_to_pyogi(&ctx, word.as_bytes());
            assert_eq!(got.as_deref(), Some(expect), "{word}");
        }
    }

    #[test]
    fn onechar_spelling_path() {
        let ctx = PronContext::empty();
        assert_eq!(
            english_word_to_pyogi(&ctx, b"KCC").as_deref(),
            Some("k9i vi vi ")
        );
        assert_eq!(
            english_word_to_pyogi(&ctx, b"AZXQ").as_deref(),
            Some("9i j9t_ 9Gs_ kyu ")
        );
        let long = "abcdefghijklmnopqrstuvwxyz";
        let got = english_word_to_pyogi(&ctx, long.as_bytes()).unwrap();
        assert!(got.starts_with("9i "), "{got}");
    }

    #[test]
    fn english_prosess_oracle() {
        let cases = [
            ("hello", "h EH l OW "),
            ("computer", "k AA m p y UW t ER "),
            ("wifi", "w AY f IH "),
            ("card", "k AA r d "),
            ("are", "AA r "),
            ("azxq", "AE z k s k "),
            ("don", "d AH n "),
            ("t", "t IY "),
            ("s", "EH z "),
            ("xl", "k s l "),
            ("xyz", "k s IH z "),
            ("dr.", "d AA k t ER "),
            ("mr.", "m IH s t ER "),
            ("mrs.", "m IH s AH s "),
            ("phd.", "p IY EY t CH d IY "),
            ("1st", "f ER s t "),
            ("21st", "t w EH n t IY f ER s t "),
            ("123", "w AH n h AH n d r EH d t w EH n t IY TH r IY "),
            ("2.5", "t UW p OY n t f AY v "),
            ("a-b", "EY b IY "),
            ("A.B", "EY p IY r IY AA d b IY "),
            ("don't", "d AH n k w OW t t IY "),
        ];
        for (input, expect) in cases {
            let got = english_prosess(input.as_bytes());
            assert_eq!(&got, expect, "{input}");
        }
    }
}
