<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { lots, ncrs, deviations } from '$lib/api/tauri.js';
	import { MobileHeader } from '$lib/components/layout/index.js';
	import { StatusBadge } from '$lib/components/common/index.js';
	import { CheckCircle, Circle, Pause, Play, Flag, AlertTriangle, Plus, ChevronDown, ChevronUp, FileText, ShieldAlert, GitBranch } from 'lucide-svelte';

	let lot = $state<Record<string, unknown> | null>(null);
	let loading = $state(true);
	let nextStep = $state<number | null>(null);
	let expandedStep = $state<number | null>(null);
	let stepUpdateLoading = $state(false);
	let linkedNcrs = $state<Record<string, unknown>[]>([]);
	let linkedDeviations = $state<Record<string, unknown>[]>([]);

	let lotId = $derived($page.params.id);

	// Execution step types matching the real entity structure
	interface WorkInstructionRef {
		id: string;
		revision?: number;
	}

	interface ExecutionStep {
		process?: string;
		process_revision?: number;
		status?: string;
		operator?: string;
		operator_email?: string;
		started_date?: string;
		completed_date?: string;
		notes?: string;
		signature_verified?: boolean;
		commit_sha?: string;
		data?: Record<string, unknown>;
		work_instructions_used?: WorkInstructionRef[];
		wi_step_executions?: Array<Record<string, unknown>>;
	}

	// WI step execution state
	let wiStepExpanded = $state<string | null>(null);
	let wiStepOperator = $state('');
	let wiStepNumber = $state(1);
	let wiStepComplete = $state(false);
	let wiStepLoading = $state(false);

	let steps = $derived<ExecutionStep[]>((lot?.execution as ExecutionStep[]) ?? []);
	let completedCount = $derived(steps.filter(s => s.status === 'completed').length);
	let progress = $derived(steps.length > 0 ? Math.round((completedCount / steps.length) * 100) : 0);

	onMount(async () => {
		await loadLot();
	});

	async function loadLot() {
		loading = true;
		try {
			const [lotData, next] = await Promise.all([
				lots.get(lotId),
				lots.getNextStep(lotId)
			]);
			lot = lotData as Record<string, unknown>;
			nextStep = next;

			// Fetch linked NCRs and deviations
			const links = lot?.links as Record<string, unknown> | undefined;
			const ncrIds = (links?.ncrs as string[]) ?? [];
			const devIds = (links?.deviations as string[]) ?? [];

			const [fetchedNcrs, fetchedDevs] = await Promise.all([
				Promise.all(ncrIds.map(id => ncrs.get(id).catch(() => null))),
				Promise.all(devIds.map(id => deviations.get(id).catch(() => null)))
			]);

			linkedNcrs = fetchedNcrs.filter(Boolean) as Record<string, unknown>[];
			linkedDeviations = fetchedDevs.filter(Boolean) as Record<string, unknown>[];
		} finally {
			loading = false;
		}
	}

	async function updateStep(index: number, status: string) {
		stepUpdateLoading = true;
		try {
			await lots.updateStep(lotId, index, { status });
			await loadLot();
			expandedStep = null;
		} finally {
			stepUpdateLoading = false;
		}
	}

	async function handleWorkflowAction(action: string) {
		try {
			switch (action) {
				case 'hold': await lots.putOnHold(lotId); break;
				case 'resume': await lots.resume(lotId); break;
				case 'complete': await lots.complete(lotId); break;
				case 'force_complete': await lots.forceComplete(lotId); break;
			}
			await loadLot();
		} catch (e) {
			console.error('Workflow action failed:', e);
		}
	}

	function getStepStatusIcon(status: string | undefined, index: number) {
		if (status === 'completed') return CheckCircle;
		if (status === 'in_progress') return Play;
		if (status === 'skipped') return Pause;
		return Circle;
	}

	async function executeWiStep(processIndex: number, wiId: string) {
		wiStepLoading = true;
		try {
			await lots.executeWiStep(lotId, {
				work_instruction_id: wiId,
				step_number: wiStepNumber,
				process_index: processIndex,
				operator: wiStepOperator || 'Unknown',
				complete: wiStepComplete
			});
			wiStepExpanded = null;
			wiStepOperator = '';
			wiStepNumber = 1;
			wiStepComplete = false;
			await loadLot();
		} catch (e) {
			console.error('Failed to execute WI step:', e);
		} finally {
			wiStepLoading = false;
		}
	}
</script>

<MobileHeader
	title={lot?.title as string ?? 'Loading...'}
	subtitle={lot?.lot_number ? `Lot #${lot.lot_number}` : undefined}
	backHref="/lots"
/>

{#if loading}
	<div class="loading-container">
		<div class="loading-spinner"></div>
	</div>
{:else if lot}
	<div class="lot-detail">
		<!-- Status & Progress Card -->
		<div class="progress-card">
			<div class="progress-top">
				<StatusBadge status={lot.lot_status as string} />
				<span class="progress-text">{completedCount}/{steps.length} steps</span>
			</div>
			<div class="progress-bar-container">
				<div class="progress-bar" style="width: {progress}%"></div>
			</div>
			<span class="progress-pct">{progress}% complete</span>
		</div>

		<!-- Quick Actions -->
		<div class="quick-actions">
			{#if lot.lot_status === 'in_progress'}
				<button class="action-btn warning" onclick={() => handleWorkflowAction('hold')}>
					<Pause size={16} /> Hold
				</button>
				<button class="action-btn success" onclick={() => handleWorkflowAction('complete')}>
					<Flag size={16} /> Complete
				</button>
			{:else if lot.lot_status === 'on_hold'}
				<button class="action-btn primary" onclick={() => handleWorkflowAction('resume')}>
					<Play size={16} /> Resume
				</button>
			{/if}
			<a href="/quality/ncrs/new?lotId={lotId}" class="action-btn destructive">
				<AlertTriangle size={16} /> New NCR
			</a>
		</div>

		<!-- Steps -->
		<section class="section">
			<h2 class="section-title">Steps</h2>
			{#if steps.length === 0}
				<p class="empty-text">No steps defined</p>
			{:else}
				<div class="steps-list">
					{#each steps as step, index}
						{@const isNext = index === nextStep}
						{@const Icon = getStepStatusIcon(step.status, index)}
						<button
							class="step-card"
							class:is-next={isNext}
							class:completed={step.status === 'completed'}
							onclick={() => expandedStep = expandedStep === index ? null : index}
						>
							<div class="step-main">
								<div class="step-icon" class:completed={step.status === 'completed'} class:active={isNext}>
									<Icon size={18} />
								</div>
								<div class="step-info">
									<span class="step-num">Step {index + 1}</span>
									<span class="step-title">{step.process || 'Unnamed step'}</span>
									{#if step.operator}
										<span class="step-meta">{step.operator}</span>
									{/if}
								</div>
								{#if expandedStep === index}
									<ChevronUp size={16} class="step-chevron" />
								{:else}
									<ChevronDown size={16} class="step-chevron" />
								{/if}
							</div>

							{#if expandedStep === index}
								<div class="step-expanded" onclick={(e) => e.stopPropagation()}>
									{#if step.started_date || step.completed_date}
										<div class="step-dates">
											{#if step.started_date}
												<span class="step-meta">Started: {new Date(step.started_date).toLocaleDateString()}</span>
											{/if}
											{#if step.completed_date}
												<span class="step-meta">Completed: {new Date(step.completed_date).toLocaleDateString()}</span>
											{/if}
										</div>
									{/if}
									{#if step.notes}
										<p class="step-notes">{step.notes}</p>
									{/if}
									<div class="step-actions">
										{#if step.status !== 'completed'}
											<button
												class="step-action-btn success"
												onclick={() => updateStep(index, 'completed')}
												disabled={stepUpdateLoading}
											>
												<CheckCircle size={16} /> Complete Step
											</button>
										{/if}
										{#if step.status !== 'in_progress' && step.status !== 'completed'}
											<button
												class="step-action-btn primary"
												onclick={() => updateStep(index, 'in_progress')}
												disabled={stepUpdateLoading}
											>
												<Play size={16} /> Start Step
											</button>
										{/if}
									</div>
									{#if step.work_instructions_used && step.work_instructions_used.length > 0}
										<div class="wi-section">
											<span class="wi-label">Work Instructions</span>
											<div class="wi-list">
												{#each step.work_instructions_used as wi}
													<button
														class="wi-btn"
														class:expanded={wiStepExpanded === `${index}-${wi.id}`}
														onclick={() => wiStepExpanded = wiStepExpanded === `${index}-${wi.id}` ? null : `${index}-${wi.id}`}
													>
														<FileText size={14} />
														<span>{wi.id}{wi.revision != null ? ` (rev ${wi.revision})` : ''}</span>
													</button>
													{#if wiStepExpanded === `${index}-${wi.id}`}
														<div class="wi-exec-form" onclick={(e) => e.stopPropagation()}>
															<input type="text" placeholder="Operator" class="wi-input" bind:value={wiStepOperator} />
															<input type="number" placeholder="Step #" min={1} class="wi-input" bind:value={wiStepNumber} />
															<label class="wi-checkbox">
																<input type="checkbox" bind:checked={wiStepComplete} />
																<span>Complete</span>
															</label>
															<button class="step-action-btn primary" onclick={() => executeWiStep(index, wi.id)} disabled={wiStepLoading}>
																{wiStepLoading ? "Executing..." : "Execute"}
															</button>
														</div>
													{/if}
												{/each}
											</div>
										</div>
									{/if}
								</div>
							{/if}
						</button>
					{/each}
				</div>
			{/if}
		</section>

		<!-- NCRs -->
		{#if linkedNcrs.length > 0}
			<section class="section">
				<h2 class="section-title">
					<AlertTriangle size={16} />
					Non-Conformances ({linkedNcrs.length})
				</h2>
				<div class="linked-list">
					{#each linkedNcrs as ncr}
						<a href="/quality/ncrs/{ncr.id}" class="linked-card ncr">
							<div class="linked-card-header">
								<StatusBadge status={ncr.ncr_status as string} />
								{#if ncr.severity}
									<span class="severity-tag {ncr.severity}">{ncr.severity}</span>
								{/if}
							</div>
							<span class="linked-card-title">{ncr.title}</span>
							{#if ncr.ncr_number}
								<span class="linked-card-meta">{ncr.ncr_number}</span>
							{/if}
						</a>
					{/each}
				</div>
			</section>
		{/if}

		<!-- Deviations -->
		{#if linkedDeviations.length > 0}
			<section class="section">
				<h2 class="section-title">
					<GitBranch size={16} />
					Process Deviations ({linkedDeviations.length})
				</h2>
				<div class="linked-list">
					{#each linkedDeviations as dev}
						<a href="/more/deviations/{dev.id}" class="linked-card deviation">
							<div class="linked-card-header">
								<StatusBadge status={dev.dev_status as string} />
								{#if dev.risk_level}
									<span class="risk-tag {dev.risk_level}">{dev.risk_level}</span>
								{/if}
							</div>
							<span class="linked-card-title">{dev.title}</span>
							<div class="linked-card-footer">
								{#if dev.deviation_number}
									<span class="linked-card-meta">{dev.deviation_number}</span>
								{/if}
								{#if dev.expiration_date}
									<span class="linked-card-meta">Expires: {new Date(dev.expiration_date as string).toLocaleDateString()}</span>
								{/if}
							</div>
						</a>
					{/each}
				</div>
			</section>
		{/if}

		<!-- Info -->
		<section class="section">
			<h2 class="section-title">Details</h2>
			<div class="info-grid">
				{#if lot.quantity}
					<div class="info-item">
						<span class="info-label">Quantity</span>
						<span class="info-value">{lot.quantity}</span>
					</div>
				{/if}
				{#if lot.author}
					<div class="info-item">
						<span class="info-label">Author</span>
						<span class="info-value">{lot.author}</span>
					</div>
				{/if}
				{#if lot.created}
					<div class="info-item">
						<span class="info-label">Created</span>
						<span class="info-value">{new Date(lot.created as string).toLocaleDateString()}</span>
					</div>
				{/if}
				{#if lot.start_date}
					<div class="info-item">
						<span class="info-label">Started</span>
						<span class="info-value">{new Date(lot.start_date as string).toLocaleDateString()}</span>
					</div>
				{/if}
			</div>
		</section>
	</div>
{/if}

<style>
	.lot-detail {
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 20px;
		padding-bottom: 32px;
	}

	.progress-card {
		background: linear-gradient(135deg, var(--theme-card), color-mix(in oklch, var(--theme-primary) 5%, var(--theme-card)));
		border: 1px solid var(--theme-border);
		border-radius: 16px;
		padding: 20px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.progress-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.progress-text {
		font-size: 14px;
		font-weight: 600;
		color: var(--theme-muted-foreground);
	}

	.progress-bar-container {
		height: 8px;
		background-color: var(--theme-muted);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-bar {
		height: 100%;
		background: linear-gradient(90deg, var(--theme-primary), var(--theme-success));
		border-radius: 4px;
		transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.progress-pct {
		font-size: 12px;
		color: var(--theme-muted-foreground);
		text-align: right;
	}

	.quick-actions {
		display: flex;
		gap: 8px;
		overflow-x: auto;
	}

	.action-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		border-radius: 12px;
		font-size: 13px;
		font-weight: 600;
		border: 1px solid var(--theme-border);
		background-color: var(--theme-card);
		color: var(--theme-foreground);
		cursor: pointer;
		white-space: nowrap;
		text-decoration: none;
		transition: transform 0.1s ease;
	}

	.action-btn:active { transform: scale(0.95); }
	.action-btn.primary { background-color: var(--theme-primary); color: var(--theme-primary-foreground); border-color: var(--theme-primary); }
	.action-btn.success { background-color: var(--theme-success); color: white; border-color: var(--theme-success); }
	.action-btn.warning { background-color: var(--theme-warning); color: black; border-color: var(--theme-warning); }
	.action-btn.destructive { background-color: color-mix(in oklch, var(--theme-error) 15%, transparent); color: var(--theme-error); border-color: var(--theme-error); }

	.section { display: flex; flex-direction: column; gap: 10px; }
	.section-title { font-size: 16px; font-weight: 700; padding: 0 4px; display: flex; align-items: center; gap: 6px; }

	.steps-list { display: flex; flex-direction: column; gap: 6px; }

	.step-card {
		display: flex;
		flex-direction: column;
		width: 100%;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		padding: 14px 16px;
		cursor: pointer;
		text-align: left;
		color: inherit;
		transition: all 0.15s ease;
	}

	.step-card.is-next {
		border-color: var(--theme-primary);
		box-shadow: 0 0 0 1px var(--theme-primary), 0 4px 12px color-mix(in oklch, var(--theme-primary) 15%, transparent);
	}

	.step-card:active { transform: scale(0.98); }

	.step-main { display: flex; align-items: center; gap: 12px; }

	.step-icon {
		width: 36px;
		height: 36px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--theme-muted);
		color: var(--theme-muted-foreground);
		flex-shrink: 0;
	}

	.step-icon.completed { background-color: var(--theme-success); color: white; }
	.step-icon.active { background-color: var(--theme-primary); color: var(--theme-primary-foreground); }

	.step-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
	.step-num { font-size: 11px; font-weight: 600; color: var(--theme-muted-foreground); text-transform: uppercase; letter-spacing: 0.05em; }
	.step-title { font-size: 14px; font-weight: 600; }
	.step-meta { font-size: 12px; color: var(--theme-muted-foreground); }

	:global(.step-chevron) { color: var(--theme-muted-foreground); flex-shrink: 0; }

	.step-expanded {
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid var(--theme-border);
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.step-dates { display: flex; gap: 16px; flex-wrap: wrap; }
	.step-notes { font-size: 13px; color: var(--theme-muted-foreground); line-height: 1.5; }

	.step-actions { display: flex; gap: 8px; }

	.step-action-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 10px 16px;
		border-radius: 10px;
		font-size: 13px;
		font-weight: 600;
		border: none;
		cursor: pointer;
		flex: 1;
		justify-content: center;
		transition: transform 0.1s ease;
	}

	.step-action-btn:active { transform: scale(0.95); }
	.step-action-btn:disabled { opacity: 0.5; }
	.step-action-btn.success { background-color: var(--theme-success); color: white; }
	.step-action-btn.primary { background-color: var(--theme-primary); color: var(--theme-primary-foreground); }

	.info-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 8px;
	}

	.info-item {
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 12px;
		padding: 12px 14px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.info-label { font-size: 11px; font-weight: 600; color: var(--theme-muted-foreground); text-transform: uppercase; letter-spacing: 0.05em; }
	.info-value { font-size: 15px; font-weight: 600; }

	.wi-section { display: flex; flex-direction: column; gap: 8px; padding-top: 10px; border-top: 1px solid var(--theme-border); }
	.wi-label { font-size: 11px; font-weight: 600; color: var(--theme-muted-foreground); text-transform: uppercase; letter-spacing: 0.05em; }
	.wi-list { display: flex; flex-direction: column; gap: 6px; }
	.wi-btn {
		display: flex; align-items: center; gap: 6px; padding: 8px 12px; border-radius: 8px; font-size: 12px; font-weight: 600;
		border: 1px solid var(--theme-border); background-color: var(--theme-card); color: var(--theme-foreground); cursor: pointer;
	}
	.wi-btn.expanded { border-color: var(--theme-primary); background-color: color-mix(in oklch, var(--theme-primary) 5%, var(--theme-card)); }
	.wi-exec-form { display: flex; flex-direction: column; gap: 8px; padding: 10px; border-radius: 8px; background: var(--theme-muted); }
	.wi-input { width: 100%; padding: 8px 10px; border: 1px solid var(--theme-border); border-radius: 8px; font-size: 13px; background: var(--theme-card); }
	.wi-checkbox { display: flex; align-items: center; gap: 6px; font-size: 13px; cursor: pointer; }

	.linked-list { display: flex; flex-direction: column; gap: 8px; }

	.linked-card {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 14px 16px;
		background-color: var(--theme-card);
		border: 1px solid var(--theme-border);
		border-radius: 14px;
		text-decoration: none;
		color: inherit;
		transition: transform 0.1s ease;
	}

	.linked-card:active { transform: scale(0.98); }

	.linked-card.ncr { border-left: 3px solid var(--theme-error); }
	.linked-card.deviation { border-left: 3px solid var(--theme-warning); }

	.linked-card-header { display: flex; align-items: center; gap: 8px; }
	.linked-card-title { font-size: 14px; font-weight: 600; line-height: 1.3; }
	.linked-card-meta { font-size: 12px; color: var(--theme-muted-foreground); }
	.linked-card-footer { display: flex; gap: 12px; flex-wrap: wrap; }

	.severity-tag, .risk-tag {
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		padding: 2px 8px;
		border-radius: 6px;
	}

	.severity-tag.minor, .risk-tag.low { background-color: color-mix(in oklch, var(--theme-success) 15%, transparent); color: var(--theme-success); }
	.severity-tag.major, .risk-tag.medium { background-color: color-mix(in oklch, var(--theme-warning) 15%, transparent); color: var(--theme-warning); }
	.severity-tag.critical, .risk-tag.high { background-color: color-mix(in oklch, var(--theme-error) 15%, transparent); color: var(--theme-error); }

	.empty-text { font-size: 13px; color: var(--theme-muted-foreground); padding: 16px; text-align: center; }

	.loading-container { display: flex; justify-content: center; padding: 64px 16px; }
	.loading-spinner { width: 32px; height: 32px; border: 3px solid var(--theme-border); border-top-color: var(--theme-primary); border-radius: 50%; animation: spin 0.8s linear infinite; }
	@keyframes spin { to { transform: rotate(360deg); } }
</style>
