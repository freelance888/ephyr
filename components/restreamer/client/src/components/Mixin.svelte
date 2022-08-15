<script lang="js">
  import { mutation } from 'svelte-apollo';
  import { showError } from '../utils/util';
  import Volume from './common/Volume.svelte';
  import Url from './common/Url.svelte';

  export let value;
  export let restream_id;
  export let output_id;
  export let mutations;
  export let activeSidechainId;

  const tuneDelayMutation = mutation(mutations.TuneDelay);
  const tuneSidechainMutation = mutation(mutations.TuneSidechain);

  let delay = 0;
  let sidechain = false;
  let isSidechainDisabled = false;

  $: {
    // Trigger Svelte reactivity watching.
    value.delay = value.delay;
    value.sidechain = value.sidechain;
    isSidechainDisabled = !!activeSidechainId && activeSidechainId !== value.id;
    // Move `sidechain` and `delay` to a separate function to omit triggering this
    // block when they are changed, as we're only interested in `value` changes
    // here.
    update_delay();
    update_sidechain();
  }

  function update_delay() {
    delay = value.delay / 1000;
  }

  function update_sidechain() {
    sidechain = value.sidechain;
  }

  async function tuneDelay() {
    const variables = {
      restream_id,
      output_id,
      mixin_id: value.id,
      delay: Math.round(delay * 1000),
    };
    try {
      await tuneDelayMutation({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  async function tuneSidechain() {
    const variables = {
      restream_id,
      output_id,
      mixin_id: value.id,
      sidechain: sidechain,
    };
    try {
      await tuneSidechainMutation({ variables });
    } catch (e) {
      showError(e.message);
    }
  }

  function hideIdentity(rawUrl) {
    let url = new URL(rawUrl);
    if (url.searchParams.get('identity')) {
      url.searchParams.delete('identity');
      url.searchParams.set('identity', '*****');
    }

    return url.toString();
  }
</script>

<template>
  <div class="mixin">
    <i class="fas fa-wave-square" title="Mixed audio" />
    <Url url={hideIdentity(value.src)} />
    <Volume
      volume={value.volume}
      {restream_id}
      {output_id}
      {mutations}
      max={value.src.startsWith('ts://') ? 1000 : 200}
      mixin_id={value.id}
    />
    <div class="options">
      <i class="far fa-clock" title="Delay" />
      <input
        class="uk-input"
        type="number"
        min="0"
        step="0.1"
        bind:value={delay}
        on:change={tuneDelay}
      />
      <span>s</span>
      <i class="fad fa-link" title="Sidechain" />
      <input
        class="uk-checkbox"
        type="checkbox"
        bind:checked={sidechain}
        disabled={isSidechainDisabled}
        on:change={tuneSidechain}
      />
    </div>
  </div>
</template>

<style lang="stylus">
  .fa-wave-square, .fa-clock, .fa-link
    font-size: 10px
    color: #d9d9d9

  .mixin
    margin-top: 6px

  .options
    padding-left: 17px
    font-size: 10px

    .uk-input
      height: auto
      width: 40px
      padding: 0
      border: none
      margin-top: -2px
      text-align: right

    .uk-checkbox
      height: 10px
      width: 10px
      padding: 0
      margin-top: -2px
      text-align: right
</style>
