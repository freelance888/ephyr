<script lang="js">
  import { createGraphQlClient, isYoutubeVideo } from '../utils/util';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './common/Shell.svelte';
  import Output from './Output.svelte';
  import { Output as Mix, TuneVolume, TuneDelay } from '../../api/mix.graphql';
  import YoutubePlayer from './common/YoutubePlayer.svelte';

  const mutations = { TuneVolume, TuneDelay };

  const gqlClient = createGraphQlClient(
    '/api-mix',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;

  const urlParams = new URLSearchParams(window.location.search);
  const output_id = urlParams.get('output');
  const restream_id = urlParams.get('id');

  const mix = subscribe(Mix, {
    errorPolicy: 'all',
    variables: {
      outputId: output_id,
      restreamId: restream_id,
    },
  });

  let title = document.title;
  $: document.title = (isOnline ? '' : '🔴  ') + title;

  $: error = $mix && $mix.error;
  $: isLoading = !isOnline || $mix.loading;
  $: canRenderMainComponent = isOnline && $mix.data;
  $: output = $mix.data && $mix.data.output;
</script>

<template>
  <Shell {canRenderMainComponent} {isLoading} {error}>
    <div slot="main" class="main">
      {#if !output}
        <section class="uk-section uk-section-muted no-output">
          <div class="uk-card-default uk-padding-small uk-text-center">
            There is no output found
          </div>
        </section>
      {:else}
        <section class="uk-section uk-section-muted single-output">
          <Output {restream_id} value={output} {mutations} />
        </section>
        {#if isYoutubeVideo(output.previewUrl)}
          <section class="uk-section uk-section-muted video-player">
            <YoutubePlayer {restream_id} preview_url={output.previewUrl} />
          </section>
        {/if}
      {/if}
    </div>
  </Shell>
</template>

<style lang="stylus">
  .main
    max-width: 960px
    padding-top: 20px

  .no-output
    padding: 20px

  .single-output
    margin-top: 20px
    padding: 10px 20px 20px 20px

    :global(.volume input)
      width: 90% !important

  .video-player
      @extend .single-output
      max-height: 800px
      min-height: 150px
</style>
