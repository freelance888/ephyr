<script lang="js">
  import Restream, { value } from '../Restream.svelte';
  import Confirm from '../Confirm.svelte';
  import { showError } from '../util';
  import { mutation } from 'svelte-apollo';
  import {
    EnableAllOutputsOfRestreams,
    DisableAllOutputsOfRestreams,
  } from '../api/graphql/client.graphql';

  const enableAllOutputsOfRestreamsMutation = mutation(
    EnableAllOutputsOfRestreams
  );
  const disableAllOutputsOfRestreamsMutation = mutation(
    DisableAllOutputsOfRestreams
  );

  export let state;
  export let info;

  $: totalInputCount = $state.data.allRestreams.length;
  $: totalOutputsCount = $state.data.allRestreams.reduce(
    (sum, input) => (sum += input.outputs.length),
    0
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
</script>

<template>
  <section class="uk-section-muted toolbar">
    <span class="label">ALL</span>
    <div class="uk-grid uk-grid-small">
      <div class="uk-width-1-2@m uk-width-1-3@s">
        <span class="toolbar-label total-inputs-label"
          >INPUTS: {totalInputCount}</span
        >
      </div>

      <span class="uk-width-expand toolbar-label"
        >OUTPUTS: {totalOutputsCount}</span
      >
      <div class="buttons-section uk-width-auto uk-flex-right">
        <Confirm let:confirm>
          <button
            class="uk-button uk-button-default"
            title="Start all outputs of all inputs"
            on:click={() => confirm(enableAllOutputsOfRestreams)}
            ><span class="uk-visible@m">Start All</span><span
              class="uk-hidden@m">Start</span
            ></button
          >
          <span slot="title">Start all outputs</span>
          <span slot="description"
            >Are you sure you want to start all outputs of all restreams?
          </span>
          <span slot="confirm">Start</span>
        </Confirm>

        <Confirm let:confirm>
          <button
            class="uk-button uk-button-default"
            title="Start all outputs of all restreams"
            on:click={() => confirm(disableAllOutputsOfRestreams)}
            value=""
            ><span class="uk-visible@m">Stop All</span><span class="uk-hidden@m"
              >Stop</span
            ></button
          >
          <span slot="title">Stop all outputs</span>
          <span slot="description"
            >Are you sure you want to stop all outputs of all restreams?
          </span>
          <span slot="confirm">Stop</span>
        </Confirm>
      </div>
    </div>
  </section>

  {#each $state.data.allRestreams as restream}
    <Restream public_host={$info.data.info.publicHost} value={restream} />
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

</style>
