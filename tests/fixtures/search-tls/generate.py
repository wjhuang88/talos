"""Offline public fixture generator; emits JSON, never persists the CA signing key.

Requires cryptography for regeneration only, not fixture/test execution.
Positive validity is deliberately fixed; renew/review before 2027-10-01.
"""
import datetime
import ipaddress
import json
import sys
from pathlib import Path

from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.x509.oid import ExtendedKeyUsageOID, NameOID


if sys.flags.optimize:
    raise SystemExit("run without -O: fixture metadata assertions must remain enabled")


def date(year, month=1, day=1):
    return datetime.datetime(year, month, day, tzinfo=datetime.timezone.utc)


if sys.argv[1:] == ["--check"]:
    directory = Path(__file__).parent
    key = serialization.load_pem_private_key(
        (directory / "PUBLIC-TEST-ONLY-leaf-key.pem").read_bytes(), password=None
    )
    root = x509.load_pem_x509_certificate((directory / "root.pem").read_bytes())
    root.verify_directly_issued_by(root)
    constraints = root.extensions.get_extension_for_class(x509.BasicConstraints).value
    assert constraints.ca and constraints.path_length == 0
    assert root.extensions.get_extension_for_class(x509.KeyUsage).value.key_cert_sign
    assert root.not_valid_before_utc == date(2020)
    assert root.not_valid_after_utc == date(2045)
    assert root.public_key().curve.name == "secp256r1"
    assert root.signature_algorithm_oid == x509.SignatureAlgorithmOID.ECDSA_WITH_SHA256
    assert root.signature_hash_algorithm.name == "sha256"
    identifier = root.extensions.get_extension_for_class(x509.SubjectKeyIdentifier).value
    assert identifier == x509.SubjectKeyIdentifier.from_public_key(root.public_key())
    now = datetime.datetime.now(datetime.timezone.utc)
    assert root.not_valid_before_utc <= now < root.not_valid_after_utc
    for name in ("leaf.pem", "host-only.pem", "expired.pem"):
        cert = x509.load_pem_x509_certificate((directory / name).read_bytes())
        cert.verify_directly_issued_by(root)
        authority = cert.extensions.get_extension_for_class(x509.AuthorityKeyIdentifier).value
        assert authority.key_identifier == identifier.digest
        assert cert.signature_hash_algorithm.name == "sha256"
        assert cert.signature_algorithm_oid == x509.SignatureAlgorithmOID.ECDSA_WITH_SHA256
        assert not cert.extensions.get_extension_for_class(x509.BasicConstraints).value.ca
        assert cert.extensions.get_extension_for_class(x509.KeyUsage).value.digital_signature
        assert ExtendedKeyUsageOID.SERVER_AUTH in cert.extensions.get_extension_for_class(x509.ExtendedKeyUsage).value
        assert cert.public_key().public_numbers() == key.public_key().public_numbers()
        assert cert.public_key().curve.name == "secp256r1"
        assert (cert.not_valid_after_utc - cert.not_valid_before_utc).days <= 398
        san = cert.extensions.get_extension_for_class(x509.SubjectAlternativeName).value
        assert san.get_values_for_type(x509.DNSName) == ["search-fixture.invalid"]
        assert san.get_values_for_type(x509.IPAddress) == ([] if name == "host-only.pem" else [ipaddress.ip_address("127.0.0.1")])
        if name == "expired.pem":
            assert cert.not_valid_after_utc == date(2021) < now
        else:
            assert cert.not_valid_before_utc == date(2026, 10, 1)
            assert cert.not_valid_after_utc == date(2027, 10, 1)
            assert cert.not_valid_before_utc <= now < cert.not_valid_after_utc
    print("public TLS fixture metadata/signatures/key/SAN/date checks passed")
    sys.exit(0)
if sys.argv[1:]:
    raise SystemExit("usage: generate.py [--check]")


ca_key = ec.generate_private_key(ec.SECP256R1())
leaf_key = serialization.load_pem_private_key(
    (Path(__file__).parent / "PUBLIC-TEST-ONLY-leaf-key.pem").read_bytes(), password=None
)
issuer = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "Talos PUBLIC TEST ONLY Search Fixture CA")])
subject = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "PUBLIC TEST ONLY search-fixture.invalid")])
root = (
    x509.CertificateBuilder().subject_name(issuer).issuer_name(issuer)
    .public_key(ca_key.public_key()).serial_number(1)
    .not_valid_before(date(2020)).not_valid_after(date(2045))
    .add_extension(x509.BasicConstraints(ca=True, path_length=0), critical=True)
    .add_extension(x509.KeyUsage(False, False, False, False, False, True, True, False, False), critical=True)
    .add_extension(x509.SubjectKeyIdentifier.from_public_key(ca_key.public_key()), critical=False)
    .sign(ca_key, hashes.SHA256())
)


def leaf(serial, expired=False, host_only=False):
    start = date(2020) if expired else date(2026, 10, 1)
    end = date(2021) if expired else date(2027, 10, 1)
    assert (end - start).days <= 398
    names = [x509.DNSName("search-fixture.invalid")]
    if not host_only:
        names.append(x509.IPAddress(ipaddress.ip_address("127.0.0.1")))
    return (
        x509.CertificateBuilder().subject_name(subject).issuer_name(issuer)
        .public_key(leaf_key.public_key()).serial_number(serial)
        .not_valid_before(start).not_valid_after(end)
        .add_extension(x509.BasicConstraints(ca=False, path_length=None), critical=True)
        .add_extension(x509.SubjectAlternativeName(names), critical=False)
        .add_extension(x509.ExtendedKeyUsage([ExtendedKeyUsageOID.SERVER_AUTH]), critical=False)
        .add_extension(x509.KeyUsage(True, False, False, False, False, False, False, False, False), critical=True)
        .add_extension(x509.AuthorityKeyIdentifier.from_issuer_public_key(ca_key.public_key()), critical=False)
        .sign(ca_key, hashes.SHA256())
    )


certificates = {
    "root.pem": root,
    "leaf.pem": leaf(2),
    "expired.pem": leaf(3, expired=True),
    "host-only.pem": leaf(4, host_only=True),
}
print(json.dumps({name: cert.public_bytes(serialization.Encoding.PEM).decode("ascii")
                  for name, cert in certificates.items()}))
