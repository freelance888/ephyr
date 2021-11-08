<script lang="js">
  import { mutation } from 'svelte-apollo';
  import { showError } from '../utils/util';
  import Volume from './Volume.svelte';
  import Url from './common/Url.svelte';

  export let value;
  export let restream_id;
  export let output_id;
  export let mutations;

  const tuneDelayMutation = mutation(mutations.TuneDelay);
  const tuneVolumeMutation = mutation(mutations.TuneVolume);

  let level = 100;
  let muted = false;
  let delay = 0;
  $: {
    // Trigger Svelte reactivity watching.
    value.delay = value.delay;
    // Move `volume` and `delay` to a separate function to omit triggering this
    // block when they are changed, as we're only interested in `value` changes
    // here.
    update_delay();
  }

  function update_delay() {
    delay = value.delay / 1000;
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
</script>

<template>
  <div class="mixin">
    <i class="fas fa-wave-square" title="Mixed audio" />
    <Url url={value.src} />
    <Volume volume={value.volume} {restream_id} {output_id} {mutations} max={value.src.startsWith('ts://') ? 1000 : 200} mixin_id={value.id}/>
    <div class="delay">
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
    </div>
  </div>
</template>

<style lang="stylus">
  .fa-wave-square, .fa-clock
    font-size: 10px
    color: #d9d9d9

  .mixin
    margin-top: 6px

  .delay
    padding-left: 17px
    font-size: 10px

    .uk-input
      height: auto
      width: 40px
      padding: 0
      border: none
      margin-top: -2px
      text-align: right
</style>
