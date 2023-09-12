// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import type { SuiSignTransactionBlockInput } from '@mysten/wallet-standard';
import type { SuiSignTransactionBlockOutput } from '@mysten/wallet-standard';
import type { UseMutationOptions } from '@tanstack/react-query';
import { useMutation } from '@tanstack/react-query';
import { useWalletContext } from '../../components/WalletProvider.js';
import { walletMutationKeys } from '../../constants/walletMutationKeys.js';
import {
	WalletFeatureNotSupportedError,
	WalletNoAccountSelectedError,
	WalletNotConnectedError,
} from '../..//errors/walletErrors.js';
import type { PartialBy } from '../../types/utilityTypes.js';

type UseSignTransactionBlockArgs = PartialBy<SuiSignTransactionBlockInput, 'account'>;
type UseSignTransactionBlockResult = SuiSignTransactionBlockOutput;

type UseSignTransactionBlockMutationOptions = Omit<
	UseMutationOptions<UseSignTransactionBlockResult, Error, UseSignTransactionBlockArgs, unknown>,
	'mutationFn'
>;

/**
 * Mutation hook for prompting the user to sign a transaction block.
 */
export function useSignTransactionBlock({
	mutationKey,
	...mutationOptions
}: UseSignTransactionBlockMutationOptions = {}) {
	const { currentWallet, currentAccount } = useWalletContext();

	return useMutation({
		mutationKey: walletMutationKeys.signTransactionBlock(mutationKey),
		mutationFn: async (signTransactionBlockArgs) => {
			if (!currentWallet) {
				throw new WalletNotConnectedError('No wallet is connected.');
			}

			const signerAccount = signTransactionBlockArgs.account ?? currentAccount;
			if (!signerAccount) {
				throw new WalletNoAccountSelectedError(
					'No wallet account is selected to sign the personal message with.',
				);
			}

			const signTransactionBlockFeature = currentWallet.features['sui:signTransactionBlock'];
			if (!signTransactionBlockFeature) {
				throw new WalletFeatureNotSupportedError(
					"This wallet doesn't support the `signTransactionBlock` feature.",
				);
			}

			return await signTransactionBlockFeature.signTransactionBlock({
				...signTransactionBlockArgs,
				account: signerAccount,
			});
		},
		...mutationOptions,
	});
}
