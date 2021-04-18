<script lang="js">
  import Restream from '../Restream.svelte';
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
  <nav class="uk-section-default toolbar">
    <div class="uk-text-right">
      <Confirm let:confirm>
        <button
          class="uk-button uk-button-default"
          title="Enable all outputs of all inputs"
          on:click={() => confirm(enableAllOutputsOfRestreams)}
          >Enable All</button
        >
        <span slot="title">Enable all outputs</span>
        <span slot="description"
          >Are you sure you want to enable all outputs of all inputs?
        </span>
        <span slot="confirm">Enable</span>
      </Confirm>

      <Confirm let:confirm>
        <button
          class="uk-button uk-button-default"
          title="Enable all outputs of all inputs"
          on:click={() => confirm(disableAllOutputsOfRestreams)}
          >Disable All</button
        >
        <span slot="title">Disable all outputs</span>
        <span slot="description"
          >Are you sure you want to disable all outputs of all inputs?
        </span>
        <span slot="confirm">Disable</span>
      </Confirm>
    </div>
  </nav>

  {#each $state.data.allRestreams as restream}
    <Restream public_host={$info.data.info.publicHost} value={restream} />
  {/each}
</template>

<style lang="stylus">
  .toolbar
    margin-top: 20px;
</style>
