// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { act, renderHook, waitFor } from '@testing-library/react';
import { createWalletProviderContextWrapper, registerMockWallet } from '../test-utils.js';
import { useConnectWallet, useWallet } from 'dapp-kit/src/index.js';
import type { Mock } from 'vitest';
import { createMockAccount } from '../mocks/mockAccount.js';
import type { StandardEventsOnMethod } from '@mysten/wallet-standard';

describe('WalletProvider', () => {
	test('the correct wallet and account information is returned on initial render', () => {
		const wrapper = createWalletProviderContextWrapper();
		const { result } = renderHook(() => useWallet(), { wrapper });

		expect(result.current).toStrictEqual({
			accounts: [],
			currentAccount: null,
			wallets: [],
			currentWallet: null,
			connectionStatus: 'disconnected',
		});
	});

	test('the list of wallets is ordered correctly by preference', () => {
		const { unregister: unregister1 } = registerMockWallet({ walletName: 'Mock Wallet 1' });
		const { unregister: unregister2 } = registerMockWallet({ walletName: 'Mock Wallet 2' });
		const { unregister: unregister3 } = registerMockWallet({ walletName: 'Mock Wallet 3' });

		const wrapper = createWalletProviderContextWrapper({
			preferredWallets: ['Mock Wallet 2', 'Mock Wallet 1'],
		});
		const { result } = renderHook(() => useWallet(), { wrapper });
		const walletNames = result.current.wallets.map((wallet) => wallet.name);

		expect(walletNames).toStrictEqual(['Mock Wallet 2', 'Mock Wallet 1', 'Mock Wallet 3']);

		act(() => {
			unregister1();
			unregister2();
			unregister3();
		});
	});

	test('the unsafe burner wallet is registered when enableUnsafeBurner is set', async () => {
		const wrapper = createWalletProviderContextWrapper({
			enableUnsafeBurner: true,
		});
		const { result } = renderHook(() => useWallet(), { wrapper });
		const walletNames = result.current.wallets.map((wallet) => wallet.name);

		expect(walletNames).toStrictEqual(['Unsafe Burner Wallet']);
	});

	test('unregistered wallets are removed from the list of wallets', async () => {
		const { unregister: unregister1 } = registerMockWallet({ walletName: 'Mock Wallet 1' });
		const { unregister: unregister2 } = registerMockWallet({ walletName: 'Mock Wallet 2' });
		const { unregister: unregister3 } = registerMockWallet({ walletName: 'Mock Wallet 3' });

		const wrapper = createWalletProviderContextWrapper();
		const { result } = renderHook(() => useWallet(), { wrapper });

		act(() => unregister2());

		const walletNames = result.current.wallets.map((wallet) => wallet.name);
		expect(walletNames).toStrictEqual(['Mock Wallet 1', 'Mock Wallet 3']);

		act(() => {
			unregister1();
			unregister3();
		});
	});

	test('the list of wallets is correctly filtered by required features', () => {
		const { unregister: unregister1 } = registerMockWallet({
			walletName: 'Mock Wallet 1',
			additionalFeatures: {
				'my-dapp:super-cool-feature': {
					version: '1.0.0',
					superCoolFeature: () => {},
				},
			},
		});
		const { unregister: unregister2 } = registerMockWallet({ walletName: 'Mock Wallet 2' });

		const wrapper = createWalletProviderContextWrapper({
			requiredFeatures: ['my-dapp:super-cool-feature'],
		});
		const { result } = renderHook(() => useWallet(), { wrapper });
		const walletNames = result.current.wallets.map((wallet) => wallet.name);

		expect(walletNames).toStrictEqual(['Mock Wallet 1']);

		act(() => {
			unregister1();
			unregister2();
		});
	});

	test('accounts are properly updated when changed from a wallet', async () => {
		const { unregister, mockWallet } = registerMockWallet({
			walletName: 'Mock Wallet 1',
			accounts: [createMockAccount(), createMockAccount(), createMockAccount()],
		});

		// Simulate the number of accounts changing as soon as the change event is registered.
		const onMock = mockWallet.features['standard:events'].on as Mock;
		onMock.mockImplementationOnce((...args: Parameters<StandardEventsOnMethod>) => {
			const [_, eventCallback] = args;
			eventCallback({
				accounts: [...mockWallet.accounts.slice(1), createMockAccount()],
			});
		});

		const wrapper = createWalletProviderContextWrapper();
		const { result } = renderHook(
			() => ({
				connectWallet: useConnectWallet(),
				walletInfo: useWallet(),
			}),
			{ wrapper },
		);

		result.current.connectWallet.mutate({ wallet: mockWallet });
		await waitFor(() => expect(result.current.connectWallet.isSuccess).toBe(true));

		// The active account the user was on was deleted, so we should expect that the user's
		// new active account is the first wallet account in the new list.
		expect(result.current.walletInfo.currentAccount).toBeTruthy();
		expect(result.current.walletInfo.currentAccount!.address).toBe(mockWallet.accounts[1].address);
		expect(result.current.walletInfo.accounts).toHaveLength(3);

		act(() => unregister());
	});
});
