<script lang="js">
  import { saveOrCloseByKeys } from '../utils/directives.util';
  import { onDestroy, onMount } from 'svelte';
  import { get, writable } from 'svelte/store';
  import { mutation } from 'svelte-apollo';
  import { AddClient, RemoveClient } from '../../api/dashboard.graphql';

  const addClientMutation = mutation(AddClient);
  const removeClientMutation = mutation(RemoveClient);

  export let visible = false;
  export let hosts = [];

  let submitable = false;
  let invalidSpec;

  let initialHostsJson = '';
  const hostsJson = writable('');

  onMount(() => {
    if (hosts.length) {
      initialHostsJson = JSON.stringify(hosts, null, 2);
      hostsJson.set(initialHostsJson);
    }
  });

  onDestroy(hostsJson.subscribe((v) => validateSpec(v)));
  onDestroy(
    hostsJson.subscribe((v) => {
      try {
        submitable =
          (initialHostsJson === '' && v.trim() !== '') ||
          JSON.stringify(JSON.parse(v)) !==
            JSON.stringify(JSON.parse(initialHostsJson));
      } catch (e) {
        submitable = false;
      }
    })
  );

  function validateSpec(spec) {
    if (spec.trim() === '') {
      invalidSpec = null;
      return;
    }
    try {
      JSON.parse(spec);
      invalidSpec = null;
    } catch (e) {
      invalidSpec = 'Failed to parse JSON: ' + e.message;
    }
  }

  async function submit() {
    if (!submitable) return;

    try {
      await removeAllHosts();
      await addHosts();
      close();
    } catch (e) {
      invalidSpec = 'Failed to apply JSON: ' + e.message;
    }
  }

  async function addHosts() {
    const newHosts = JSON.parse(get(hostsJson));
    for (const host of newHosts) {
      await addClientMutation({ variables: { client_id: host } });
    }
  }

  async function removeAllHosts() {
    for (const host of hosts) {
      await removeClientMutation({ variables: { client_id: host } });
    }
  }

  function close() {
    visible = false;
  }

  const jsonPlaceholderText = `Array of hosts :
[
  "http://localhost/",
  "http://192.168.0.2/"
]
`;
</script>

<template>
  <div class="uk-modal uk-open" use:saveOrCloseByKeys={{ close: close }}>
    <div class="uk-modal-dialog uk-modal-body">
      <h2 class="uk-modal-title">Export or import hosts as JSON</h2>
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
          bind:value={$hostsJson}
          on:change={() => validateSpec($hostsJson)}
          placeholder={jsonPlaceholderText}
        />
        {#if !!invalidSpec}
          <span class="uk-form-danger spec-err">{invalidSpec}</span>
        {/if}
      </fieldset>

      <button
        class="uk-button uk-button-primary"
        disabled={!submitable}
        on:click={async () => await submit()}
        title="Replaces existing list of hosts with the given JSON"
        >Replace</button
      >
    </div>
  </div>
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
</style>
