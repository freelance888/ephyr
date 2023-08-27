<script lang="js">
  import Fa from 'svelte-fa';
  import { faLock, faLockOpen } from '@fortawesome/free-solid-svg-icons';

  import { mutation } from 'svelte-apollo';

  import Confirm from './common/Confirm.svelte';
  import StatusFilter from './common/StatusFilter';
  import { escapeRegExp, isFailoverInput, showError } from '../utils/util';
  import {
    DisableAllOutputsOfRestreams,
    EnableAllOutputsOfRestreams,
    UpdateOrder
  } from '../../api/client.graphql';
  import OutputModal from '../modals/OutputModal.svelte';
  import PasswordModal from '../modals/PasswordModal.svelte';
  import {
    getAggregatedStreamsData,
    toggleFilterStatus,
  } from '../utils/filters.util';
  import {
    statusesList,
    STREAM_ERROR,
    STREAM_WARNING,
    streamStatusList,
  } from '../utils/constants';
  import { onDestroy } from 'svelte';
  import Restream from './Restream.svelte';
  import cloneDeep from 'lodash/cloneDeep';
  import { dndzone } from 'svelte-dnd-action';

  const enableAllOutputsOfRestreamsMutation = mutation(
    EnableAllOutputsOfRestreams
  );
  const disableAllOutputsOfRestreamsMutation = mutation(
    DisableAllOutputsOfRestreams
  );

  const updateOrderMutation = mutation(UpdateOrder);

  export let state;
  export let info;
  export let files;

  const searchQueryKey = 'search';
  const filterQueryKey = 'filter_by';
  let params = new URLSearchParams(location.search);
  const searchString = params.get(searchQueryKey);
  const filterBy = params.get(filterQueryKey);
  let searchText = decodeURIComponent(searchString ? searchString : '');

  let searchInInputs = searchText ? filterBy.includes('input') : true;
  let searchInOutputs = searchText ? filterBy.includes('output') : false;

  $: orderedRestreams = undefined;
  $: allReStreams = [];
  $: aggregatedStreamsData = getAggregatedStreamsData(allReStreams);

  $: globalInputsFilters = [];
  $: globalOutputsFilters = [];
  $: hasActiveFilters = globalInputsFilters.length;

  $: {
      allReStreams = getFilteredRestreams(
        searchText,
        orderedRestreams ?? $state.data.allRestreams,
        searchInInputs,
        searchInOutputs
      );
  }

  $: dragDisabled = true;

  $: inputsSortMode = false;

  $: outputsSortMode = false;

  const isReStreamVisible = (restream) => {
    const hasInputFilter = globalInputsFilters.includes(
      restream.input.endpoints[0].status
    );

    const hasStreamsWarnings =
      globalInputsFilters.includes(STREAM_WARNING) &&
      aggregatedStreamsData.endpointsStreamsStatus[STREAM_WARNING].includes(
        restream.input.id
      );

    const hasStreamsErrors =
      globalInputsFilters.includes(STREAM_ERROR) &&
      aggregatedStreamsData.endpointsStreamsStatus[STREAM_ERROR].includes(
        restream.input.id
      );

    return hasInputFilter || hasStreamsWarnings || hasStreamsErrors;
  };

  const getStreamStatusFilterTitle = (status) => {
    return status === STREAM_WARNING
      ? 'Inputs with inconsistencies in streams params'
      : status === STREAM_ERROR
      ? 'Inputs with errors on getting streams params'
      : '';
  };

  const storeSearchTextInQueryParams = () => {
    if (searchText) {
      const queryParams = new URLSearchParams();
      queryParams.set(searchQueryKey, encodeURIComponent(searchText));
      appendFilterByParams(queryParams);
      history.replaceState(null, null, '?' + queryParams.toString());
    } else {
      history.replaceState(null, null, '/');
    }
  };

  const appendFilterByParams = (queryParams) => {
    const searchInParam = searchInInputs && 'input';
    const searchOutParam = searchInOutputs && 'output';
    const filterBy = (searchInParam || searchOutParam) && 'filter_by';
    const params = [];
    if (filterBy) {
      searchInParam && params.push(encodeURIComponent(searchInParam));
      searchOutParam && params.push(encodeURIComponent(searchOutParam));
      queryParams.append(filterBy, params.join(','));
    }
  };

  const getFilteredRestreams = (
    substring,
    originalRestreams,
    onlyInInputs,
    onlyInOutputs
  ) => {
    storeSearchTextInQueryParams();

    if (!substring) {
      return originalRestreams;
    }

    // Case-insensitive search
    const regex = new RegExp(escapeRegExp(substring), 'i');

    return cloneDeep(originalRestreams).filter((x) => {
      let foundOutputs = [];
      if (onlyInOutputs) {
        foundOutputs = x.outputs.filter((o) => o.label && regex.test(o.label));
        if (foundOutputs.length) {
          x.outputs = foundOutputs;
        }
      }

      const hasRestreamLabel = onlyInInputs && x.label && regex.test(x.label);
      const hasInputLabel =
        onlyInInputs &&
        x.input.endpoints.filter((e) => e.label && regex.test(e.label)).length >
          0;

      const hasFailoverInputLabel =
        onlyInInputs &&
        isFailoverInput(x.input) &&
        x.input.src.inputs
          .flatMap((x) => x.endpoints)
          .filter((e) => e.label && regex.test(e.label)).length > 0;

      return (
        hasRestreamLabel ||
        hasInputLabel ||
        hasFailoverInputLabel ||
        foundOutputs.length
      );
    });
  };

  let currentHash = undefined;
  onDestroy(
    info.subscribe((i) => {
      if (i.data) {
        const newHash = i.data.info.passwordHash;
        if (currentHash === undefined) {
          currentHash = newHash;
        } else if (!!newHash && newHash !== currentHash) {
          window.location.reload();
        }

        const title = i.data.info.title;
        document.title = title || ' Ephyr re-streamer';
      }
    })
  );

  async function enableAllOutputsOfRestreams() {
    try {
      await enableAllOutputsOfRestreamsMutation();
    } catch (e) {
      showError(e.message);
    }
  }

  async function disableAllOutputsOfRestreams() {
    try {
      await disableAllOutputsOfRestreamsMutation();
    } catch (e) {
      showError(e.message);
    }
  }

  let openPasswordOutputModal = false;

  function onChangeSearchInInput() {
    if (!searchInInputs && !searchInOutputs) {
      searchInInputs = true;
    }
  }

  function onChangeSearchInOutputs() {
    if (!searchInInputs && !searchInOutputs) {
      searchInOutputs = true;
    }
  }

  function handleSort(e) {
    orderedRestreams = e.detail.items;
    dragDisabled = true;
  }

  async function onDrop(e) {
    const ids = e.detail.items.map((x) => x.id);
    handleSort(e);

    await updateOrder(ids);
    setTimeout(() => {
      orderedRestreams = undefined;
    }, 2000)
  }

  function onDragStarted(e) {
    dragDisabled = e.details;
  }

  async function updateOrder(ids) {
    try {
      const variables = { ids };
      await updateOrderMutation({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

</script>

<template>
  <OutputModal />

  <section class="uk-section-muted toolbar">
    <span class="section-label">Filters</span>
    <div class="uk-grid uk-grid-small uk-flex-middle">
      <div>
        <span class="toolbar-label">
          INPUTS:

          {#each statusesList as status (status)}
            <StatusFilter
              {status}
              count={aggregatedStreamsData.inputsCountByStatus[status]}
              active={globalInputsFilters.includes(status)}
              handleClick={() =>
                (globalInputsFilters = toggleFilterStatus(
                  globalInputsFilters,
                  status
                ))}
            />
          {/each}
        </span>
      </div>
      <div class="uk-margin-small-left">
        <span class="toolbar-label">
          STREAMS:
          {#each streamStatusList as status (status)}
            <StatusFilter
              {status}
              count={aggregatedStreamsData.endpointsStreamsStatus[status]
                .length}
              active={globalInputsFilters.includes(status)}
              title={getStreamStatusFilterTitle(status)}
              handleClick={() =>
                (globalInputsFilters = toggleFilterStatus(
                  globalInputsFilters,
                  status
                ))}
            />
          {/each}
        </span>
      </div>
      <div class="uk-flex-auto uk-flex-right uk-flex uk-flex-middle">
        <span class="toolbar-label"
          >OUTPUTS:

          {#each statusesList as status (status)}
            <StatusFilter
              {status}
              count={aggregatedStreamsData.outputsCountByStatus[status]}
              active={globalOutputsFilters.includes(status)}
              handleClick={() =>
                (globalOutputsFilters = toggleFilterStatus(
                  globalOutputsFilters,
                  status
                ))}
            />
          {/each}
        </span>
        {#key $info.data.info.passwordOutputHash}
          <a
            href="/"
            class="set-output-password"
            title="{!$info.data.info.passwordOutputHash
                ? 'Set'
                : 'Change'} output password"
            on:click|preventDefault={() => (openPasswordOutputModal = true)}
          >
            {#if $info.data.info.passwordOutputHash}
              <Fa icon={faLock}></Fa>
            {:else}
              <Fa icon={faLockOpen}></Fa>
            {/if}
          </a>
          {#if openPasswordOutputModal}
            <PasswordModal
              password_kind="OUTPUT"
              current_hash={$info.data.info.passwordOutputHash}
              bind:visible={openPasswordOutputModal}
            />
          {/if}
        {/key}
      </div>
      <div class="uk-margin-auto-left">
        <!-- TODO: move Confirm modals to other files -->
        <Confirm let:confirm>
          <button
            class="uk-button uk-button-default"
            data-testid="start-all-outputs"
            title="Start all outputs of all restreams"
            on:click={() => confirm(enableAllOutputsOfRestreams)}
            ><span>Start All</span>
          </button>
          <span slot="title">Start all outputs</span>
          <span slot="description"
            >This will start all outputs of all restreams.
          </span>
          <span slot="confirm">Start</span>
        </Confirm>

        <Confirm let:confirm>
          <button
            class="uk-button uk-button-default"
            data-testid="stop-all-outputs"
            title="Stop all outputs of all restreams"
            on:click={() => confirm(disableAllOutputsOfRestreams)}
            value=""><span>Stop All</span></button
          >
          <span slot="title">Stop all outputs</span>
          <span slot="description"
            >This will stop all outputs of all restreams.
          </span>
          <span slot="confirm">Stop</span>
        </Confirm>
      </div>
    </div>

    <input
      class="uk-input uk-width-1-3 uk-margin-small-top"
      bind:value={searchText}
      placeholder="Search by labels (regex)"
    />
    <button
      type="button"
      class="clear-search"
      uk-close
      on:click={() => (searchText = '')}
    />
    <div class="uk-margin-small-top">
      <label>
        <input
          class="uk-checkbox"
          bind:checked={searchInInputs}
          on:change={onChangeSearchInInput}
          type="checkbox"
        /> in inputs
      </label>
      <label>
        <input
          class="uk-checkbox uk-margin-small-left"
          bind:checked={searchInOutputs}
          on:change={onChangeSearchInOutputs}
          type="checkbox"
        /> in outputs
      </label>
    </div>
    <div>
      <button
        type="button"
        on:click={() => inputsSortMode = !inputsSortMode}
      >Sort mode</button>
      <button
        type="button"
        on:click={() => outputsSortMode = !outputsSortMode}
      >Output sort mode</button>

    </div>
  </section>

  <div
    use:dndzone={{
        items: allReStreams,
        dropTargetClasses: ['drop-target'],
        dragDisabled,
        flipDurationMs: 200,
      }}
    on:consider={handleSort}
    on:finalize={onDrop}
  >
    {#each allReStreams as restream (restream.id)}
        <Restream
          public_host={$info.data.info.publicHost}
          on:dragStarted={onDragStarted}
          inputsSortMode={inputsSortMode}
          outputsSortMode={outputsSortMode}
          value={restream}
          hidden={globalInputsFilters?.length && !isReStreamVisible(restream)}
          {globalOutputsFilters}
          {files}
        />
    {:else}
      <div
        class="uk-section uk-section-muted uk-section-xsmall uk-padding uk-text-center"
      >
        <div>
          There are no Inputs. You can add it by clicking <b>+INPUT</b> button.
        </div>
      </div>
    {/each}
  </div>
</template>

<style lang="stylus">
  :global(.drop-target) {
    outline: none !important;
  }

  .set-output-password
    margin-left: 10px;
    display: inline-block
    color: var(--primary-text-color)

    &:hover
      color: #444

  .clear-search
    position: relative
    left: -30px;
    top: 4px;
</style>
