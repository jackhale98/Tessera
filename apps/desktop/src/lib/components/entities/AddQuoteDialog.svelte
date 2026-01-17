<script lang="ts">
	import { Dialog, Button, Input, Label, Select } from '$lib/components/ui';
	import { entities } from '$lib/api';
	import { Receipt, Plus, Trash2 } from 'lucide-svelte';

	interface Props {
		open: boolean;
		componentId: string;
		componentTitle?: string;
		onClose: () => void;
		onCreated?: (quoteId: string) => void;
	}

	let { open = $bindable(), componentId, componentTitle, onClose, onCreated }: Props = $props();

	// Form state
	let title = $state('');
	let supplierId = $state('');
	let supplierName = $state('');
	let currency = $state<'USD' | 'EUR' | 'GBP' | 'CNY' | 'JPY'>('USD');
	let moq = $state<number | null>(null);
	let defaultLeadTimeDays = $state<number | null>(null);
	let saving = $state(false);
	let error = $state<string | null>(null);

	// Price breaks - qty:price:leadTime
	interface PriceBreak {
		min_qty: number;
		unit_price: number;
		lead_time_days: number | null;
	}
	let priceBreaks = $state<PriceBreak[]>([{ min_qty: 1, unit_price: 0, lead_time_days: null }]);

	// Available suppliers from the project
	let suppliers = $state<Array<{ id: string; name: string }>>([]);
	let loadingSuppliers = $state(false);

	// Load suppliers when dialog opens
	$effect(() => {
		if (open) {
			loadSuppliers();
			// Reset form
			title = '';
			supplierId = '';
			supplierName = '';
			currency = 'USD';
			moq = null;
			defaultLeadTimeDays = null;
			priceBreaks = [{ min_qty: 1, unit_price: 0, lead_time_days: null }];
			error = null;
		}
	});

	async function loadSuppliers() {
		loadingSuppliers = true;
		try {
			const result = await entities.list('SUP', { limit: 100 });
			suppliers = result.items.map((s) => ({
				id: s.id,
				name: s.title
			}));
		} catch (e) {
			console.error('Failed to load suppliers:', e);
		} finally {
			loadingSuppliers = false;
		}
	}

	function addPriceBreak() {
		// Get the last break's qty as starting point for new one
		const lastBreak = priceBreaks[priceBreaks.length - 1];
		const newQty = lastBreak ? Math.max(lastBreak.min_qty * 10, 100) : 100;
		priceBreaks = [...priceBreaks, { min_qty: newQty, unit_price: 0, lead_time_days: null }];
	}

	function removePriceBreak(index: number) {
		if (priceBreaks.length > 1) {
			priceBreaks = priceBreaks.filter((_, i) => i !== index);
		}
	}

	function updatePriceBreak(index: number, field: keyof PriceBreak, value: number | null) {
		priceBreaks = priceBreaks.map((pb, i) => {
			if (i === index) {
				return { ...pb, [field]: value };
			}
			return pb;
		});
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (!title.trim()) {
			error = 'Quote title is required';
			return;
		}

		if (!supplierId && !supplierName.trim()) {
			error = 'Supplier is required. Select an existing supplier or enter a new name.';
			return;
		}

		// Validate price breaks
		const validBreaks = priceBreaks.filter((pb) => pb.unit_price > 0);
		if (validBreaks.length === 0) {
			error = 'At least one price break with a price greater than 0 is required';
			return;
		}

		saving = true;
		error = null;

		try {
			// Create supplier if not selecting existing one
			let finalSupplierId = supplierId;
			if (!finalSupplierId && supplierName.trim()) {
				// Create new supplier
				const supplierData: Record<string, unknown> = {
					title: supplierName.trim(),
					status: 'draft',
					capabilities: [],
					certifications: [],
					links: {},
					created: new Date().toISOString(),
					author: 'TDT User', // TODO: Get from project config
					entity_revision: 1
				};
				finalSupplierId = await entities.save('SUP', supplierData);
			}

			// Build price breaks array - sort by quantity and filter valid ones
			const sortedBreaks = validBreaks
				.map((pb) => ({
					min_qty: pb.min_qty,
					unit_price: pb.unit_price,
					lead_time_days: pb.lead_time_days ?? defaultLeadTimeDays ?? undefined
				}))
				.sort((a, b) => a.min_qty - b.min_qty);

			// Create the quote entity
			const quoteData: Record<string, unknown> = {
				title: title.trim(),
				supplier: finalSupplierId,
				component: componentId,
				currency: currency,
				price_breaks: sortedBreaks,
				quote_status: 'received',
				status: 'draft',
				links: {},
				created: new Date().toISOString(),
				author: 'TDT User', // TODO: Get from project config
				entity_revision: 1
			};

			if (moq) {
				quoteData.moq = moq;
			}

			if (defaultLeadTimeDays) {
				quoteData.lead_time_days = defaultLeadTimeDays;
			}

			const quoteId = await entities.save('QUOT', quoteData);

			onCreated?.(quoteId);
			onClose();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			console.error('Failed to create quote:', e);
		} finally {
			saving = false;
		}
	}

	function handleClose() {
		error = null;
		onClose();
	}

	function formatCurrency(value: number): string {
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: currency
		}).format(value);
	}

	// Calculate total for preview
	const previewTotal = $derived(() => {
		const validBreaks = priceBreaks.filter((pb) => pb.unit_price > 0);
		if (validBreaks.length === 0) return null;
		// Show the first break's price as preview
		const firstBreak = validBreaks.sort((a, b) => a.min_qty - b.min_qty)[0];
		return firstBreak?.unit_price ?? null;
	});
</script>

<Dialog bind:open onClose={handleClose}>
	<form onsubmit={handleSubmit} class="max-h-[85vh] overflow-y-auto p-6">
		<div class="mb-6">
			<div class="flex items-center gap-3">
				<div class="flex h-10 w-10 items-center justify-center rounded-full bg-green-500/10">
					<Receipt class="h-5 w-5 text-green-600" />
				</div>
				<div>
					<h2 class="text-lg font-semibold">Add Quote</h2>
					{#if componentTitle}
						<p class="text-sm text-muted-foreground">for {componentTitle}</p>
					{/if}
				</div>
			</div>
		</div>

		{#if error}
			<div class="mb-4 rounded-md bg-destructive/10 p-3 text-sm text-destructive">
				{error}
			</div>
		{/if}

		<div class="space-y-4">
			<!-- Quote Title -->
			<div>
				<Label for="title">Quote Title *</Label>
				<Input
					id="title"
					bind:value={title}
					placeholder="e.g., Q1 2026 Quote from Acme Corp"
					class="mt-1.5"
					disabled={saving}
				/>
			</div>

			<!-- Supplier Selection -->
			<div>
				<Label for="supplier">Supplier *</Label>
				{#if loadingSuppliers}
					<div class="mt-1.5 flex h-10 items-center text-sm text-muted-foreground">
						Loading suppliers...
					</div>
				{:else if suppliers.length > 0}
					<Select
						id="supplier"
						bind:value={supplierId}
						class="mt-1.5"
						disabled={saving}
					>
						<option value="">Select existing supplier or add new below...</option>
						{#each suppliers as sup}
							<option value={sup.id}>{sup.name}</option>
						{/each}
					</Select>
				{/if}
				{#if !supplierId}
					<Input
						bind:value={supplierName}
						placeholder="Or enter new supplier name..."
						class="mt-2"
						disabled={saving || !!supplierId}
					/>
					<p class="mt-1 text-xs text-muted-foreground">
						A new supplier will be created if you enter a name here
					</p>
				{/if}
			</div>

			<!-- Currency -->
			<div>
				<Label for="currency">Currency</Label>
				<Select
					id="currency"
					bind:value={currency}
					class="mt-1.5"
					disabled={saving}
				>
					<option value="USD">USD ($)</option>
					<option value="EUR">EUR</option>
					<option value="GBP">GBP</option>
					<option value="CNY">CNY</option>
					<option value="JPY">JPY</option>
				</Select>
			</div>

			<!-- Price Breaks -->
			<div>
				<div class="flex items-center justify-between">
					<Label>Price Breaks *</Label>
					<Button
						type="button"
						variant="outline"
						size="sm"
						onclick={addPriceBreak}
						disabled={saving}
					>
						<Plus class="mr-1 h-3 w-3" />
						Add Break
					</Button>
				</div>
				<p class="mb-2 text-xs text-muted-foreground">
					Enter quantity-based pricing (e.g., 1+ @ $10, 100+ @ $8, 1000+ @ $6)
				</p>

				<div class="space-y-2">
					{#each priceBreaks as pb, index}
						<div class="flex items-center gap-2 rounded-lg border bg-muted/30 p-2">
							<div class="flex-1">
								<Label class="text-xs">Min Qty</Label>
								<Input
									type="number"
									min="1"
									value={pb.min_qty}
									onchange={(e) => updatePriceBreak(index, 'min_qty', parseInt(e.currentTarget.value) || 1)}
									class="mt-1 h-8"
									disabled={saving}
								/>
							</div>
							<div class="flex-1">
								<Label class="text-xs">Unit Price *</Label>
								<Input
									type="number"
									step="0.01"
									min="0"
									value={pb.unit_price || ''}
									onchange={(e) => updatePriceBreak(index, 'unit_price', parseFloat(e.currentTarget.value) || 0)}
									placeholder="0.00"
									class="mt-1 h-8"
									disabled={saving}
								/>
							</div>
							<div class="flex-1">
								<Label class="text-xs">Lead (days)</Label>
								<Input
									type="number"
									min="1"
									value={pb.lead_time_days ?? ''}
									onchange={(e) => updatePriceBreak(index, 'lead_time_days', e.currentTarget.value ? parseInt(e.currentTarget.value) : null)}
									placeholder="—"
									class="mt-1 h-8"
									disabled={saving}
								/>
							</div>
							{#if priceBreaks.length > 1}
								<Button
									type="button"
									variant="ghost"
									size="sm"
									class="mt-5 h-8 w-8 p-0 text-muted-foreground hover:text-destructive"
									onclick={() => removePriceBreak(index)}
									disabled={saving}
								>
									<Trash2 class="h-4 w-4" />
								</Button>
							{:else}
								<div class="mt-5 h-8 w-8"></div>
							{/if}
						</div>
					{/each}
				</div>
			</div>

			<!-- MOQ and Default Lead Time -->
			<div class="grid grid-cols-2 gap-4">
				<div>
					<Label for="moq">Minimum Order Qty</Label>
					<Input
						id="moq"
						type="number"
						min="1"
						bind:value={moq}
						placeholder="e.g., 100"
						class="mt-1.5"
						disabled={saving}
					/>
				</div>
				<div>
					<Label for="leadTime">Default Lead Time (days)</Label>
					<Input
						id="leadTime"
						type="number"
						min="1"
						bind:value={defaultLeadTimeDays}
						placeholder="e.g., 14"
						class="mt-1.5"
						disabled={saving}
					/>
					<p class="mt-1 text-xs text-muted-foreground">
						Used when break has no specific lead time
					</p>
				</div>
			</div>

			<!-- Summary Preview -->
			{#if previewTotal() !== null}
				<div class="rounded-lg bg-muted/50 p-3">
					<p class="text-sm font-medium">Quote Summary</p>
					<div class="mt-2 space-y-1 text-sm text-muted-foreground">
						{#each priceBreaks.filter(pb => pb.unit_price > 0).sort((a, b) => a.min_qty - b.min_qty) as pb}
							<div class="flex justify-between">
								<span>{pb.min_qty}+ units:</span>
								<span class="font-medium text-foreground">
									{formatCurrency(pb.unit_price)}
									{#if pb.lead_time_days}
										<span class="text-muted-foreground">({pb.lead_time_days}d)</span>
									{/if}
								</span>
							</div>
						{/each}
						{#if moq}
							<div class="mt-2 border-t pt-2">MOQ: {moq}</div>
						{/if}
					</div>
				</div>
			{/if}
		</div>

		<div class="mt-6 flex justify-end gap-3">
			<Button type="button" variant="outline" onclick={handleClose} disabled={saving}>
				Cancel
			</Button>
			<Button
				type="submit"
				disabled={saving || !title.trim() || (!supplierId && !supplierName.trim()) || priceBreaks.filter(pb => pb.unit_price > 0).length === 0}
			>
				{#if saving}
					Creating...
				{:else}
					Create Quote
				{/if}
			</Button>
		</div>
	</form>
</Dialog>
