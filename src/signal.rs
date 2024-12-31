//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

fn ignore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigIgn) }
        .expect("sush(fatal): cannot ignore signal");
}

fn restore_signal(sig: Signal) {
    unsafe { signal::signal(sig, SigHandler::SigDfl) }
        .expect("sush(fatal): cannot restore signal");
}
