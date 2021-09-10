<script lang="js">
  import { onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { mutation } from 'svelte-apollo';
  import { Import } from '../../api/client.graphql';
  import { exportModal as value } from '../stores';
  import { saveOrCloseByKeys } from '../utils/directives.util';

  const importMutation = mutation(Import);

  let submitable = false;
  onDestroy(
    value.subscribe((v) => {
      try {
        submitable =
          (v.prev_spec === '' && v.spec.trim() !== '') ||
          JSON.stringify(JSON.parse(v.spec)) !==
            JSON.stringify(JSON.parse(v.prev_spec));
      } catch (e) {
        submitable = false;
      }
    })
  );

  let invalidSpec = null;
  onDestroy(value.subscribe((v) => validateSpec(v.spec)));
  function validateSpec(v) {
    if (v.trim() === '') {
      invalidSpec = null;
      return;
    }
    try {
      JSON.parse(v);
      invalidSpec = null;
    } catch (e) {
      invalidSpec = 'Failed to parse JSON: ' + e.message;
    }
  }

  async function submit(replace) {
    if (!submitable) return;
    const v = get(value);
    const variables = { replace, spec: v.spec, restream_id: v.restream_id };
    try {
      await importMutation({ variables });
      value.close();
    } catch (e) {
      invalidSpec = 'Failed to apply JSON: ' + e.message;
    }
  }

  function close() {
    value.close();
  }
</script>

<template>
  {#if $value.visible}
    <div class="uk-modal uk-open" use:saveOrCloseByKeys={{ close: close }}>
      <div class="uk-modal-dialog uk-modal-body">
        <h2 class="uk-modal-title">Export or import as JSON</h2>
        <button
          class="uk-modal-close-outside"
          uk-close
          type="button"
          on:click={close}
        />

        <fieldset>
          <textarea
            class="uk-textarea"
            class:uk-form-danger={!!invalidSpec}
            bind:value={$value.spec}
            on:change={() => validateSpec($value.spec)}
            placeholder="JSON..."
          />
          {#if !!invalidSpec}
            <span class="uk-form-danger spec-err">{invalidSpec}</span>
          {/if}
        </fieldset>

        <button
          class="uk-button uk-button-primary"
          disabled={!submitable}
          on:click={() => submit(true)}
          title="Replaces existing definitions with the given JSON"
          >Replace</button
        >
        <button
          class="uk-button uk-button-primary"
          disabled={!submitable}
          on:click={() => submit(false)}
          title="Merges the given JSON with existing definitions without removing anything"
          >Apply</button
        >
      </div>
    </div>
  {/if}
</template>

<style lang="stylus">
  .uk-modal
    &.uk-open
      display: block

    .uk-modal-title
      font-size: 1.5rem

    fieldset
      border: none
      padding: 0

      .uk-textarea
        min-height: 200px
        resize: none

      .spec-err
        display: block
        font-size: 11px

    button + button
      margin-right: 15px
</style>
