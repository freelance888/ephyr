<svelte:options immutable={true} />

<script lang="js">
  import { mutation, getClient, subscribe } from 'svelte-apollo';

  import {
    RemoveRestream,
    DisableOutput,
    DisableAllOutputs,
    EnableAllOutputs,
    EnableOutput,
    ExportRestream,
    RemoveOutput,
    TuneDelay,
    TuneVolume,
    TuneSidechain,
    Info,
  } from '../../api/client.graphql';

  import { showError } from '../utils/util';
  import { statusesList } from '../constants/statuses';

  import { restreamModal, outputModal, exportModal } from '../stores';

  import Confirm from './common/Confirm.svelte';
  import Input from './Input.svelte';
  import Output from './Output.svelte';
  import Toggle from './common/Toggle.svelte';
  import StatusFilter from './common/StatusFilter.svelte';
  import { getReStreamOutputsCount } from '../utils/restreamHelpers.util';
  import { toggleFilterStatus } from '../utils/statusFilters.util';
  import { RestreamModel } from '../models/restream.model';
  import RestreamModal from '../modals/RestreamModal.svelte';

  const removeRestreamMutation = mutation(RemoveRestream);
  const disableAllOutputsMutation = mutation(DisableAllOutputs);
  const enableAllOutputsMutation = mutation(EnableAllOutputs);

  const gqlClient = getClient();
  const info = subscribe(Info, { errorPolicy: 'all' });

  export let public_host = 'localhost';
  // TODO: rename 'value' to 'reStream'
  export let value;
  export let globalOutputsFilters;
  export let hidden = false;

  let outputMutations = {
    DisableOutput,
    EnableOutput,
    RemoveOutput,
    TuneVolume,
    TuneDelay,
    TuneSidechain,
  };

  $: deleteConfirmation = $info.data
    ? $info.data.info.deleteConfirmation
    : true;

  $: enableConfirmation = $info.data
    ? $info.data.info.enableConfirmation
    : true;

  $: allEnabled = value.outputs.every((o) => o.enabled);
  $: toggleStatusText = allEnabled ? 'Disable' : 'Enable';

  $: hasGlobalOutputsFilters = !!globalOutputsFilters.length;
  $: reStreamOutputsCountByStatus = getReStreamOutputsCount(value);
  // NOTE: if global filters are selected, they have higher priority
  $: reStreamOutputsFilters = hasGlobalOutputsFilters
    ? globalOutputsFilters
    : [];
  $: hasActiveFilters = reStreamOutputsFilters.length;

  let openRestreamModal = false;

  function openEditRestreamModal() {
    const with_hls = value.input.endpoints.some((e) => e.kind === 'HLS');

    let pull_url = null;
    let backup = null;

    if (!!value.input.src && value.input.src.__typename === 'RemoteInputSrc') {
      pull_url = value.input.src.url;
    }

    if (
      !!value.input.src &&
      value.input.src.__typename === 'FailoverInputSrc'
    ) {
      backup = true;
      if (!!value.input.src.inputs[0].src) {
        pull_url = value.input.src.inputs[0].src.url;
      }
      if (!!value.input.src.inputs[1].src) {
        backup = value.input.src.inputs[1].src.url;
      }
    }

    restreamModal.openEdit(
      value.id,
      value.key,
      value.label,
      pull_url,
      backup,
      with_hls
    );
  }

  async function removeRestream() {
    try {
      await removeRestreamMutation({ variables: { id: value.id } });
    } catch (e) {
      showError(e.message);
    }
  }

  function openAddOutputModal() {
    outputModal.openAdd(value.id);
  }

  async function toggleAllOutputs() {
    if (value.outputs.length < 1) return;
    const variables = { restream_id: value.id };
    try {
      if (allEnabled) {
        await disableAllOutputsMutation({ variables });
      } else {
        await enableAllOutputsMutation({ variables });
      }
    } catch (e) {
      showError(e.message);
    }
  }

  async function openExportModal() {
    let resp;
    try {
      resp = await gqlClient.query({
        query: ExportRestream,
        variables: { id: value.id },
        fetchPolicy: 'no-cache',
      });
    } catch (e) {
      showError(e.message);
      return;
    }

    if (!!resp.data && !!resp.data.export) {
      exportModal.open(
        value.id,
        JSON.stringify(JSON.parse(resp.data.export), null, 2)
      );
    }
  }
</script>

<template>
  <div
    data-testid={value.label}
    class="uk-section uk-section-muted uk-section-xsmall"
    class:hidden
  >
    <div class="left-buttons-area" />
    <div class="right-buttons-area" />
    <Confirm let:confirm>
      <button
        type="button"
        class="uk-close"
        uk-close
        on:click={deleteConfirmation
          ? () => confirm(removeRestream)
          : removeRestream}
      />
      <span slot="title"
        >Removing <code>{value.key}</code> input source for re-streaming</span
      >
      <span slot="description"
        >All its outputs will be removed too. You won't be able to undone this.</span
      >
      <span slot="confirm">Remove</span>
    </Confirm>

    <button
      class="uk-button uk-button-primary uk-button-small"
      data-testid="add-output:open-modal-btn"
      on:click={openAddOutputModal}
    >
      <i class="fas fa-plus" />&nbsp;<span>Output</span>
    </button>

    <a
      class="export-import"
      href="/"
      on:click|preventDefault={openExportModal}
      title="Export/Import"
    >
      <i class="fas fa-share-square" />
    </a>

    {#if !!value.label}
      <span class="section-label">{value.label}</span>
    {/if}

    {#if value.outputs && value.outputs.length > 0}
      <span class="total">
        {#each statusesList as status (status)}
          <StatusFilter
            {status}
            count={reStreamOutputsCountByStatus[status]}
            active={reStreamOutputsFilters.includes(status)}
            disabled={hasGlobalOutputsFilters}
            title={hasGlobalOutputsFilters &&
              'Filter is disabled while global output filters are active'}
            handleClick={() =>
              (reStreamOutputsFilters = toggleFilterStatus(
                reStreamOutputsFilters,
                status
              ))}
          />
        {/each}

        <Confirm let:confirm>
          <Toggle
            data-testid="toggle-all-outputs-status"
            id="all-outputs-toggle-{value.id}"
            checked={allEnabled}
            title="{toggleStatusText} all outputs"
            confirmFn={enableConfirmation ? confirm : undefined}
            onChangeFn={toggleAllOutputs}
          />
          <span slot="title"
            >{toggleStatusText} all outputs of <code>{value.key}</code> input</span
          >
          <span slot="description">Are you sure about it?</span>
          <span slot="confirm">{toggleStatusText}</span>
        </Confirm>
      </span>
    {/if}

    <a
      class="edit-input"
      href="/"
      on:click|preventDefault={() => openRestreamModal = true}
    >
      <i class="far fa-edit" title="Edit input" />
    </a>
    {#if openRestreamModal}
      <RestreamModal public_host={$info.data.info.publicHost} bind:visible={openRestreamModal} model={new RestreamModel(value)}/>
    {/if}
    <Input
      {public_host}
      restream_id={value.id}
      restream_key={value.key}
      value={value.input}
    />
    {#if !!value.input.src && value.input.src.__typename === 'FailoverInputSrc'}
      {#each value.input.src.inputs as input}
        <Input
          {public_host}
          restream_id={value.id}
          restream_key={value.key}
          value={input}
        />
      {/each}
    {/if}

    <div class="uk-grid uk-grid-small" uk-grid>
      {#each value.outputs as output}
        <Output
          {deleteConfirmation}
          {enableConfirmation}
          {public_host}
          restream_id={value.id}
          value={output}
          hidden={hasActiveFilters &&
            !reStreamOutputsFilters.includes(output.status)}
          mutations={outputMutations}
        />
      {:else}
        <div class="uk-flex-1">
          <div class="uk-card-default uk-padding-small uk-text-center">
            There are no Outputs for current Input. You can add it by clicking <b
              >+OUTPUT</b
            > button.
          </div>
        </div>
      {/each}
    </div>
  </div>
</template>

<style lang="stylus">
  .uk-section
    position: relative
    margin-top: 20px
    padding-left: 10px
    padding-right: @padding-left

    &.hidden
      display: none

    &:hover
      .uk-close, .uk-button-small
      .edit-input, .export-import
        opacity: 1

    .uk-button-small
      float: right
      font-size: 0.7rem
      margin-top: -2px
      opacity: 0
      transition: opacity .3s ease
      &:hover
        opacity: 1

    .total
      float: right
      margin-right: 20px

    .edit-input, .export-import, .uk-close
      position: absolute
      opacity: 0
      transition: opacity .3s ease
      &:hover
        opacity: 1
    .edit-input, .export-import
      color: #666
      outline: none
      &:hover
        text-decoration: none
        color: #444
    .edit-input
      left: -25px
    .export-import
      right: -25px
    .uk-close
      right: -21px
      top: -15px

    .left-buttons-area, .right-buttons-area
      position: absolute
      width: 34px
    .left-buttons-area
      right: 100%
      top: 0
      height: 100%
    .right-buttons-area
      left: 100%
      top: -20px
      height: calc(20px + 100%)

    .uk-grid
      margin-top: 10px
      margin-left: -10px

</style>
