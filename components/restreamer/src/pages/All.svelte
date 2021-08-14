<script lang="js">
  import { mutation } from 'svelte-apollo';

  import Restream from '../Restream.svelte';
  import Confirm from '../Confirm.svelte';
  import StatusFilter from '../components/common/StatusFilter';
  import { showError } from '../util';
  import {
    EnableAllOutputsOfRestreams,
    DisableAllOutputsOfRestreams,
  } from '../api/graphql/client.graphql';
  import OutputModal from '../OutputModal.svelte';
  import PasswordModal from '../PasswordModal.svelte';
  import { getAggregatedStreamsData } from './allHelpers';
  import { statusesList } from '../constants/statuses';
  import { toggleFilterStatus } from '../utils/statusFilters.util';

  const enableAllOutputsOfRestreamsMutation = mutation(
    EnableAllOutputsOfRestreams
  );
  const disableAllOutputsOfRestreamsMutation = mutation(
    DisableAllOutputsOfRestreams
  );

  export let state;
  export let info;

  $: allReStreams = $state.data.allRestreams;
  $: aggregatedStreamsData = getAggregatedStreamsData(allReStreams);

  $: globalInputsFilters = [];
  $: globalOutputsFilters = [];
  $: hasActiveFilters = globalInputsFilters.length;
  $: filteredReStreams = hasActiveFilters
    ? allReStreams.filter((stream) =>
        globalInputsFilters.includes(stream.input.endpoints[0].status)
      )
    : allReStreams;

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
</script>

<template>
  <OutputModal />
  <section class="uk-section-muted toolbar">
    <span class="label">ALL</span>
    <div class="uk-grid uk-grid-small">
      <div class="uk-width-1-2@m uk-width-1-3@s">
        <span class="toolbar-label total-inputs-label">
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

      <div class="uk-width-expand">
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
            on:click|preventDefault={() => (openPasswordOutputModal = true)}
          >
            <i
              class="fas"
              class:fa-lock-open={!$info.data.info.passwordOutputHash}
              class:fa-lock={!!$info.data.info.passwordOutputHash}
              title="{!$info.data.info.passwordOutputHash
                ? 'Set'
                : 'Change'} output password"
            />
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
      <div class="uk-panel uk-width-auto uk-flex-right">
        <!-- TODO: move Confirm modals to other files -->
        <Confirm let:confirm>
          <button
            class="uk-button uk-button-default"
            title="Start all outputs of all restreams"
            on:click={() => confirm(enableAllOutputsOfRestreams)}
            ><span class="uk-visible@m">Start All</span><span
              class="uk-hidden@m">Start</span
            ></button
          >
          <span slot="title">Start all outputs</span>
          <span slot="description"
            >This will start all outputs of all restreams.
          </span>
          <span slot="confirm">Start</span>
        </Confirm>

        <Confirm let:confirm>
          <button
            class="uk-button uk-button-default"
            title="Stop all outputs of all restreams"
            on:click={() => confirm(disableAllOutputsOfRestreams)}
            value=""
            ><span class="uk-visible@m">Stop All</span><span class="uk-hidden@m"
              >Stop</span
            ></button
          >
          <span slot="title">Stop all outputs</span>
          <span slot="description"
            >This will stop all outputs of all restreams.
          </span>
          <span slot="confirm">Stop</span>
        </Confirm>
      </div>
    </div>
  </section>

  {#each filteredReStreams as restream}
    <Restream
      public_host={$info.data.info.publicHost}
      value={restream}
      {globalOutputsFilters}
    />
  {:else}
    <div
      class="uk-section uk-section-muted uk-section-xsmall uk-padding uk-text-center"
    >
      {#if hasActiveFilters}
        There are no Inputs matching your filter criteria.
      {:else}
        <div>
          There are no Inputs. You can add it by clicking <b>+INPUT</b> button.
        </div>
      {/if}
    </div>
  {/each}
</template>

<style lang="stylus">
  .toolbar
    position: relative
    padding: 8px
    margin-top: 20px
    margin-bottom: 30px

  .toolbar-label
    line-height: 38px
    margin-left: 12px

  .label
    position: absolute
    top: -14px
    left: 0
    padding: 2px 10px
    border-top-left-radius: 4px
    border-top-right-radius: 4px
    background-color: #f8f8f8

  .set-output-password
    margin-left: 10px;
    display: inline-block
    color: var(--primary-text-color)
    &:hover
      color: #444

</style>
