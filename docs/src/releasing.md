# Releasing Simulator-Bindings

1. List Simics-Base package versions in simics-api-sys/build.rs by running:

```sh
ispm packages --list-available --json \
    | awk '/\{/,EOF' \
    | jq -r '.availablePackages[] | select(.pkgNumber == 1000) | .version' \
    | tac \
    | grep -v pre \
    | sed -ne '/6.0.163/,$ p'
```

2. Update command at bottom of simics-api-sys/build.rs with any missing
   versions from the list

3. Install all supported Simics-Base package versions (update command with
   latest versions, here 6.0.207 and 7.0.20)

`kinit; ispm packages -i 1000-6.0.{163..207} 1000-7.{0..20}.0`

4. Run the `gen-api-items.rs` script using the command in simics-api-sys/build.rs like:

```sh
./scripts/gen-simics-api-items.rs \
    -s ~/simics/simics-6.0.163 \
    -s ~/simics/simics-6.0.164 \
    -s ~/simics/simics-6.0.165 \
    -s ~/simics/simics-6.0.166 \
    -s ~/simics/simics-6.0.167 \
    -s ~/simics/simics-6.0.168 \
    -s ~/simics/simics-6.0.169 \
    -s ~/simics/simics-6.0.170 \
    -s ~/simics/simics-6.0.171 \
    -s ~/simics/simics-6.0.172 \
    -s ~/simics/simics-6.0.173 \
    -s ~/simics/simics-6.0.174 \
    -s ~/simics/simics-6.0.175 \
    -s ~/simics/simics-6.0.176 \
    -s ~/simics/simics-6.0.177 \
    -s ~/simics/simics-6.0.178 \
    -s ~/simics/simics-6.0.179 \
    -s ~/simics/simics-6.0.180 \
    -s ~/simics/simics-6.0.181 \
    -s ~/simics/simics-6.0.182 \
    -s ~/simics/simics-6.0.183 \
    -s ~/simics/simics-6.0.184 \
    -s ~/simics/simics-6.0.185 \
    -s ~/simics/simics-6.0.186 \
    -s ~/simics/simics-6.0.187 \
    -s ~/simics/simics-6.0.188 \
    -s ~/simics/simics-6.0.189 \
    -s ~/simics/simics-6.0.190 \
    -s ~/simics/simics-6.0.191 \
    -s ~/simics/simics-6.0.192 \
    -s ~/simics/simics-6.0.193 \
    -s ~/simics/simics-6.0.194 \
    -s ~/simics/simics-6.0.195 \
    -s ~/simics/simics-6.0.196 \
    -s ~/simics/simics-6.0.197 \
    -s ~/simics/simics-6.0.198 \
    -s ~/simics/simics-6.0.199 \
    -s ~/simics/simics-6.0.200 \
    -s ~/simics/simics-6.0.201 \
    -s ~/simics/simics-6.0.202 \
    -s ~/simics/simics-6.0.203 \
    -s ~/simics/simics-6.0.204 \
    -s ~/simics/simics-6.0.205 \
    -s ~/simics/simics-6.0.206 \
    -s ~/simics/simics-6.0.207 \
    -s ~/simics/simics-7.0.0 \
    -s ~/simics/simics-7.1.0 \
    -s ~/simics/simics-7.2.0 \
    -s ~/simics/simics-7.3.0 \
    -s ~/simics/simics-7.4.0 \
    -s ~/simics/simics-7.5.0 \
    -s ~/simics/simics-7.6.0 \
    -s ~/simics/simics-7.7.0 \
    -s ~/simics/simics-7.8.0 \
    -s ~/simics/simics-7.9.0 \
    -s ~/simics/simics-7.10.0 \
    -s ~/simics/simics-7.11.0 \
    -s ~/simics/simics-7.12.0 \
    -s ~/simics/simics-7.13.0 \
    -s ~/simics/simics-7.14.0 \
    -s ~/simics/simics-7.15.0 \
    -s ~/simics/simics-7.16.0 \
    -s ~/simics/simics-7.17.0 \
    -s ~/simics/simics-7.18.0 \
    -s ~/simics/simics-7.19.0 \
    -s ~/simics/simics-7.20.0 \
    -o simics-api-sys/simics_api_items.rs
```

5. Build high-level bindings for any new features that merit inclusion. Not all
   features have high-level bindings, but most do. These high level bindings
   should utilize the versioned `cfg()` directives.

7. Run check script: `./check.sh`
    - This will report issues with formatting (C and Python formatting can be ignored
      for releases, markdown and Rust issues should be fixed)
    - This will perform most checks done in CI including dependencies
    - Any dependencies that are outdated or flag vulnerabilities in audits should be
      updated
    - Any code which has breaking changes (very rare) should be fixed
