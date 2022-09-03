use crate::token::TokenType;

/// Optimizes a token stream for computational speed (not memory).
/// Primarily, this removes unneeded tokens from the stream
pub fn optimize(tokens: &mut Vec<TokenType>, level: usize) {
    let opt_levels = [|| opt_0(&mut tokens), || opt_1(&mut tokens)];
    for _ in 1..level {}
}

fn opt_0(tokens: &mut Vec<TokenType>) {
    //Remove any redundant 'Clear' tokens
    for i in (1..tokens.len()).rev() {
        match tokens[i] {
            TokenType::Clear => {
                //If there's a Clear token before this one, remove this one
                if let Some(TokenType::Clear) = tokens.get(i - 1) {
                    tokens.remove(i);
                }
            }
            _ => (),
        }
    }

    //Finally, set the index of all jump tokens (this is optimization, but must be done)
    //Also, this MUST happen after any tokens are added or removed
    for i in 0..tokens.len() {
        match &tokens[i] {
            TokenType::PreComputeJump { arg } => {
                let label: &String = arg;
                let mut index: usize = usize::MAX;

                for ib in 0..tokens.len() {
                    match &tokens[ib] {
                        TokenType::Label { arg } => {
                            if arg == label {
                                index = ib;
                                break;
                            };
                        }
                        _ => (),
                    }
                }

                //Replace the `PreComputeJump` with a new `Jump`
                tokens[i] = TokenType::Jump { arg: index };
            }
            _ => (),
        }
    }
}

fn opt_1(tokens: &mut Vec<TokenType>) {
    todo!()
}
