// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import type { SuiSignPersonalMessageInput } from '@mysten/wallet-standard';
import type { SuiSignPersonalMessageOutput } from '@mysten/wallet-standard';
import type { UseMutationOptions } from '@tanstack/react-query';
import { useMutation } from '@tanstack/react-query';
import { useWalletContext } from 'dapp-kit/src/components/wallet-provider/WalletProvider';
import {
	WalletFeatureNotSupportedError,
	WalletNotConnectedError,
} from 'dapp-kit/src/errors/walletErrors';

type UseSignPersonalMessageArgs = SuiSignPersonalMessageInput;
type UseSignPersonalMessageResult = SuiSignPersonalMessageOutput;

type UseSignPersonalMessageMutationOptions = Omit<
	UseMutationOptions<UseSignPersonalMessageResult, Error, UseSignPersonalMessageArgs, unknown>,
	'mutationKey' | 'mutationFn'
>;

// TODO: Figure out the query/mutation key story and whether or not we want to expose
// key factories from dapp-kit
function mutationKey(args: Partial<UseSignPersonalMessageArgs>) {
	return [{ scope: 'wallet', entity: 'sign-personal-message', ...args }] as const;
}

/**
 * Mutation hook for prompting the user to sign a message.
 */
export function useSignPersonalMessage({
	message,
	account,
	...mutationOptions
}: Partial<UseSignPersonalMessageArgs> & UseSignPersonalMessageMutationOptions) {
	const { currentWallet } = useWalletContext();

	return useMutation({
		mutationKey: mutationKey({ message, account }),
		mutationFn: async (personalMessageInput) => {
			if (!currentWallet) {
				throw new WalletNotConnectedError('No wallet is connected.');
			}

			const signPersonalMessageFeature = currentWallet.features['sui:signPersonalMessage'];
			if (!signPersonalMessageFeature) {
				throw new WalletFeatureNotSupportedError(
					"This wallet doesn't support the `signPersonalMessage` feature.",
				);
			}

			return await signPersonalMessageFeature.signPersonalMessage({
				...personalMessageInput,
				account: personalMessageInput.account,
			});
		},
		...mutationOptions,
	});
}
