<svelte:options immutable={true} />

<script lang="js">
  import Fa from 'svelte-fa';
  import { faEdit } from '@fortawesome/free-regular-svg-icons';
  import { faPlus } from '@fortawesome/free-solid-svg-icons';
  import { faShareSquare } from '@fortawesome/free-solid-svg-icons';
  import { dndzone } from 'svelte-dnd-action';

  import { getClient, mutation, subscribe } from 'svelte-apollo';

  import {
    DisableAllOutputs,
    DisableOutput,
    EnableAllOutputs,
    EnableOutput,
    ExportRestream,
    Info,
    RemoveOutput,
    RemoveRestream,
    TuneDelay,
    TuneSidechain,
    TuneVolume,
    CurrentlyPlayingFile,
  } from '../../api/client.graphql';

  import {
    getFullStreamUrl,
    isArrayStartWithAnother,
    isFailoverInput,
    showError,
  } from '../utils/util';
  import { statusesList } from '../utils/constants';

  import { exportModal, outputModal } from '../stores';
  import { UpdateOutputsOrder } from '../../api/client.graphql';

  import Confirm from './common/Confirm.svelte';
  import Input from './input/Input.svelte';
  import Output from './Output.svelte';
  import Toggle from './common/Toggle.svelte';
  import StatusFilter from './common/StatusFilter.svelte';
  import {
    getReStreamOutputsCount,
    toggleFilterStatus,
  } from '../utils/filters.util';
  import { RestreamModel } from '../models/restream.model';
  import RestreamModal from '../modals/RestreamModal.svelte';
  import cloneDeep from 'lodash/cloneDeep';
  import {
    getEndpointsWithDiffStreams,
    getEndpointsWithStreamsErrors,
  } from '../utils/streamInfo.util';
  import EqualizerIcon from './svg/EqualizerIcon.svelte';
  import PlaylistIcon from './svg/PlaylistIcon.svelte';
  import FileInfo from './common/FileInfo.svelte';
  import { createEventDispatcher } from 'svelte';
  import StreamInfoDiffTooltip from './common/StreamInfoDiffTooltip.svelte';

  const removeRestreamMutation = mutation(RemoveRestream);
  const disableAllOutputsMutation = mutation(DisableAllOutputs);
  const enableAllOutputsMutation = mutation(EnableAllOutputs);

  const gqlClient = getClient();
  const info = subscribe(Info, { errorPolicy: 'all' });

  const dispatch = createEventDispatcher();

  export let public_host = 'localhost';
  // TODO: rename 'value' to 'reStream'
  export let value;
  export let globalOutputsFilters;
  export let hidden = false;
  export let isFullView = false;
  export let inputsSortMode = false;
  export let outputsSortMode = false;

  let outputMutations = {
    DisableOutput,
    EnableOutput,
    RemoveOutput,
    TuneVolume,
    TuneDelay,
    TuneSidechain,
  };

  const updateOutputsOrderMutation = mutation(UpdateOutputsOrder);

  const playingFile = subscribe(CurrentlyPlayingFile, {
    variables: { id: value.id },
    errorPolicy: 'all',
  });

  $: orderedOutputs = undefined;

  $: outputs = orderedOutputs ?? value.outputs;

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

  $: hasVideos = value.playlist && value.playlist.queue.length > 0;

  $: isPlaylistPlaying = value.playlist.currentlyPlayingFile;

  $: showControls = false;

  $: streamsErrorsTooltip = getStreamErrorTooltip(value.input);

  $: streamsDiffTooltip = getStreamsDifferenceTooltip(value.input);

  $: failoverInputsCount = value.input.src?.inputs?.length ?? 0;

  $: currentlyPlayingFile =
    isPlaylistPlaying && $playingFile.data?.currentlyPlayingFile;

  $: dragDisabled = true;

  $: orderWasUpdated = true;

  $: if (value.outputs && orderedOutputs && !orderWasUpdated) {
    const storedIds = value.outputs.map((x) => x.id);
    const orderedIds = orderedOutputs.map((x) => x.id);

    if (isArrayStartWithAnother(orderedIds, storedIds)) {
      orderWasUpdated = true;
      orderedOutputs = undefined;
    }
  }

  let openRestreamModal = false;

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

  function intputStartDrag(e) {
    e.preventDefault();
    dispatch('inputsDragStarted', false);
  }

  function outputsHandleSort(e) {
    orderedOutputs = e.detail.items;
    dragDisabled = true;
  }

  async function onDropOutput(e) {
    const ids = e.detail.items.map((x) => x.id);
    outputsHandleSort(e);

    orderWasUpdated = false;
    await updateOutputsOrder(ids);
  }

  function onOutputDragStarted(e) {
    dragDisabled = e.details;
  }

  async function updateOutputsOrder(ids) {
    try {
      const variables = { ids, restreamId: value.id };
      await updateOutputsOrderMutation({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  const getStreamErrorTooltip = (input) => {
    const inputKeys = getEndpointsWithStreamsErrors(input);
    return inputKeys?.length
      ? `Can't get stream info from <strong>${inputKeys}</strong>`
      : '';
  };

  const getStreamsDifferenceTooltip = (input) => {
    const result = getEndpointsWithDiffStreams(input, currentlyPlayingFile);
    return result?.endpointsWithDiffStreams?.length
      ? `<strong>${result.endpointsWithDiffStreams.join(', ')}</strong> ${
          result.endpointsWithDiffStreams.length === 1 ? 'stream' : 'streams'
        } params ${
          result.endpointsWithDiffStreams.length === 1 ? 'differs' : 'differ'
        } from <strong>${result.firstEndpointKey}</strong> stream params`
      : '';
  };
</script>

<template>
  <div
    data-testid={value.label}
    class="uk-section uk-section-muted uk-section-xsmall"
    class:hidden
    on:mouseenter={() => (showControls = true)}
    on:mouseleave={() => (showControls = false)}
  >
    <div class="left-buttons-area" />
    <div class="right-buttons-area" />
    <Confirm let:confirm>
      <button
        type="button"
        class="uk-close"
        hidden={isFullView || outputsSortMode || inputsSortMode}
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

    <a
      class="export-import"
      hidden={isFullView || outputsSortMode || inputsSortMode}
      href="/"
      on:click|preventDefault={openExportModal}
      title="Export/Import"
    >
      <Fa icon={faShareSquare} />
    </a>

    {#if !!value.label || streamsErrorsTooltip || streamsDiffTooltip}
      <span class="section-label"
        >{value.label ?? ''}
        <StreamInfoDiffTooltip {streamsErrorsTooltip} {streamsDiffTooltip} />
      </span>
    {/if}

    <div
      class:uk-hidden={inputsSortMode || outputsSortMode}
      class="uk-float-right uk-flex uk-flex-middle"
    >
      <a
        href={getFullStreamUrl(value.id)}
        hidden={isFullView}
        target="_blank"
        rel="noreferrer"
        class="uk-text-uppercase full-view-link"
        title="Open Full Stream Page"
      >
        Full view
      </a>
      <div class="uk-flex uk-flex-middle">
        <span
          class="playlist-icon uk-margin-right uk-margin-small-left uk-flex uk-flex-middle"
          class:is-playing={isPlaylistPlaying}
          aria-hidden="true"
          hidden={!hasVideos || isFullView}
        >
          {#if isPlaylistPlaying}
            <EqualizerIcon />
          {:else}
            <PlaylistIcon />
          {/if}
        </span>
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
      </div>
      <button
        class="uk-button uk-button-primary uk-button-small"
        data-testid="add-output:open-modal-btn"
        on:click={openAddOutputModal}
      >
        <Fa icon={faPlus} />
        <span>Output</span>
      </button>
    </div>

    <span
      class:uk-hidden={!inputsSortMode}
      class="item-drag-zone uk-icon"
      uk-icon="table"
      on:mousedown={intputStartDrag}
    />
    <a
      class:uk-hidden={inputsSortMode || outputsSortMode}
      data-testid="edit-input-modal:open"
      class="edit-input"
      href="/"
      title="Edit input"
      on:click|preventDefault={() => (openRestreamModal = true)}
    >
      <Fa icon={faEdit} />
    </a>
    {#if openRestreamModal}
      <RestreamModal
        public_host={$info.data.info.publicHost}
        bind:visible={openRestreamModal}
        restream={new RestreamModel(cloneDeep(value))}
      />
    {/if}

    {#if !outputsSortMode}
      <Input
        {public_host}
        restream_id={value.id}
        restream_key={value.key}
        value={value.input}
        with_label={false}
        show_controls={showControls}
      />
      {#if isFailoverInput(value.input) && !inputsSortMode}
        {#each value.input.src.inputs as input, index}
          <Input
            {public_host}
            restream_id={value.id}
            restream_key={value.key}
            value={input}
            with_label={true}
            show_controls={showControls}
            show_move_up={failoverInputsCount > 1 && index !== 0}
            show_up_confirmation={failoverInputsCount > 1 && index === 1}
            show_move_down={failoverInputsCount > 1 &&
              index !== failoverInputsCount - 1}
          />
        {/each}
        {#if currentlyPlayingFile}
          <div class="uk-flex uk-flex-middle currently-playing-file">
            <div class="playlist-file-icon">
              <EqualizerIcon />
            </div>
            <div class="file-info">
              <FileInfo file={currentlyPlayingFile} />
            </div>
          </div>
        {/if}
      {/if}
    {/if}

    {#if !inputsSortMode}
      <div
        class="uk-grid uk-grid-small"
        use:dndzone={{
          items: outputs,
          type: 'output',
          dropTargetClasses: ['drop-target'],
          dropFromOthersDisabled: true,
          dragDisabled,
          flipDurationMs: 200,
        }}
        on:consider={outputsHandleSort}
        on:finalize={onDropOutput}
      >
        {#each outputs as output (output.id)}
          <Output
            {deleteConfirmation}
            {enableConfirmation}
            {public_host}
            {outputsSortMode}
            on:outputDragStarted={onOutputDragStarted}
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
    {/if}
  </div>
</template>

<style lang="stylus">
  :global(.drop-target) {
    outline: none !important;
  }

  .uk-section
    position: relative
    margin-top: 20px
    padding-left: 10px
    padding-right: @padding-left

    &.hidden
      display: none

    &:hover
      .uk-close, .edit-input, .export-import, .uk-button-small, .full-view-link, .item-drag-zone
        opacity: 1

    .uk-button-small
      margin-left: 16px
      font-size: 0.7rem
      margin-top: -2px
      opacity: 0
      transition: opacity .3s ease

    .edit-input, .export-import, .uk-close, .item-drag-zone
      position: absolute
      opacity: 0
      transition: opacity .3s ease

      &:hover
        opacity: 1

    .full-view-link
      font-size: 0.8rem
      transition: opacity .3s ease
      margin-right: 8px
      opacity: 0

    .edit-input, .export-import, .item-drag-zone
      color: #666
      outline: none

      &:hover
        text-decoration: none
        color: #444

    .item-drag-zone
      cursor: grab
      left: -25px

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

    .currently-playing-file
      margin-top: 4px
      margin-left: 40px

      .file-info
        margin-left: 6px

    .playlist-file-icon
      color: var(--success-color)
      :global(svg)
        width: 16px
        height: 16px

    .playlist-icon
      &.is-playing
        color: var(--success-color)
      :global(svg)
        width: 20px
        height: 20px
</style>
