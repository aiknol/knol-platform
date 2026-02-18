import { Metadata } from 'next';
import { pageTitle } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Terms of Service'),
  description: 'Terms of Service for Knol — agreement governing use of our platform.',
};

export default function TermsPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold text-dark-50 mb-2">Terms of Service</h1>
        <p className="text-dark-300 mb-12">Effective Date: February 15, 2026</p>

        {/* Introduction */}
        <section className="mb-12">
          <p className="text-dark-300 text-lg leading-relaxed">
            These Terms of Service ("Terms") govern your access to and use of Knol's website, API, SDKs, and services
            (collectively, the "Service"). By accessing or using the Service, you agree to be bound by these Terms.
            If you do not agree to these Terms, you may not use the Service.
          </p>
        </section>

        {/* Acceptance */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">1. Acceptance of Terms</h2>
          <p className="text-dark-300">
            By using the Service, you represent that you are at least 18 years old and have the legal capacity to enter
            into these Terms. If you are using the Service on behalf of an organization or entity, you represent that
            you have the authority to bind that organization to these Terms.
          </p>
        </section>

        {/* Services Description */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">2. Description of Services</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              Knol is a context engineering platform that enables you to:
            </p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>Store, manage, and search persistent memories for AI applications</li>
              <li>Build and query knowledge graphs from your data</li>
              <li>Integrate memory and context into LLM applications via REST API and SDKs</li>
              <li>Monitor and audit memory operations through the admin dashboard</li>
            </ul>
            <p className="mt-4">
              We reserve the right to modify, suspend, or discontinue the Service (or any feature or portion thereof)
              at any time, with or without notice. We shall not be liable to you if any features are unavailable or
              discontinued.
            </p>
          </div>
        </section>

        {/* User Accounts */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">3. User Accounts</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              To use the Service, you may need to create an account. You agree to:
            </p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>Provide accurate, current, and complete information during registration</li>
              <li>Maintain the confidentiality of your password and API keys</li>
              <li>Notify us immediately of any unauthorized access to your account</li>
              <li>Be responsible for all activities that occur under your account</li>
              <li>Use the Service only for lawful purposes</li>
            </ul>
            <p className="mt-4">
              We reserve the right to suspend or terminate your account if we believe you have violated these Terms
              or engaged in fraudulent, abusive, or harmful activity.
            </p>
          </div>
        </section>

        {/* API Usage */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">4. API Usage and Rate Limiting</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              When using the Knol API, you agree to:
            </p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>Not exceed the rate limits specified for your subscription plan</li>
              <li>Not use the API in a manner that could harm or degrade the Service</li>
              <li>Not reverse-engineer, decompile, or attempt to discover the source code</li>
              <li>Not scrape, cache, or store API responses in violation of these Terms</li>
              <li>Not use the API for competitive analysis or to develop competing products</li>
            </ul>
            <p className="mt-4">
              Violation of these restrictions may result in immediate suspension of your API access and account termination.
            </p>
          </div>
        </section>

        {/* Intellectual Property */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">5. Intellectual Property Rights</h2>
          <div className="space-y-4 text-dark-300">
            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">5.1 Knol IP</h3>
              <p>
                The Service, including all software, documentation, designs, and content created by Knol, is protected
                by copyright, trademark, and other intellectual property laws. You are granted a limited, non-exclusive,
                non-transferable license to use the Service in accordance with these Terms. You may not modify, copy,
                distribute, or sublicense any portion of the Service.
              </p>
            </div>
            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">5.2 Your Data</h3>
              <p>
                You retain all rights to any data, content, and memories you upload to the Service. You grant us
                a worldwide, non-exclusive license to use your data solely to provide and improve the Service.
                We will not sell or share your data with third parties except as necessary to provide the Service
                or as required by law.
              </p>
            </div>
            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">5.3 Open Source</h3>
              <p>
                Knol is open source and available under the License specified in the repository. Your use of
                Knol as an open-source project is governed by the applicable open-source license, not these Terms.
              </p>
            </div>
          </div>
        </section>

        {/* Limitations of Liability */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">6. Limitation of Liability</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              TO THE FULLEST EXTENT PERMITTED BY LAW:
            </p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>THE SERVICE IS PROVIDED "AS IS" WITHOUT WARRANTIES OF ANY KIND, EXPRESS OR IMPLIED</li>
              <li>WE DISCLAIM ALL WARRANTIES, INCLUDING MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND NON-INFRINGEMENT</li>
              <li>WE SHALL NOT BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, OR CONSEQUENTIAL DAMAGES</li>
              <li>OUR TOTAL LIABILITY FOR ANY CLAIMS SHALL NOT EXCEED THE FEES YOU PAID IN THE PAST 12 MONTHS</li>
              <li>SOME JURISDICTIONS DO NOT ALLOW LIMITATION OF LIABILITY, SO THIS MAY NOT APPLY TO YOU</li>
            </ul>
          </div>
        </section>

        {/* Indemnification */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">7. Indemnification</h2>
          <p className="text-dark-300">
            You agree to indemnify, defend, and hold harmless Knol and its officers, directors, employees, and agents
            from any claims, damages, losses, or expenses (including reasonable attorneys' fees) arising from or related to:
            (a) your use of the Service; (b) your data or content uploaded to the Service; (c) your violation of these Terms;
            or (d) your infringement of any third-party rights.
          </p>
        </section>

        {/* Termination */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">8. Termination</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              You may terminate your account at any time by contacting us. We may terminate or suspend your account
              and access to the Service immediately, without notice, if:
            </p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>You violate any provision of these Terms</li>
              <li>We determine in our sole discretion that your usage is harmful or abusive</li>
              <li>You engage in fraudulent, illegal, or unethical behavior</li>
              <li>We cease operations or decide to discontinue the Service</li>
            </ul>
            <p className="mt-4">
              Upon termination, your right to use the Service will immediately cease. We may retain your data for
              a limited period as required by law, then delete it permanently.
            </p>
          </div>
        </section>

        {/* Governing Law */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">9. Governing Law and Jurisdiction</h2>
          <p className="text-dark-300">
            These Terms shall be governed by and construed in accordance with the laws of the United States,
            without regard to its conflict of law provisions. You agree to submit to the exclusive jurisdiction of
            the state and federal courts located in the United States for resolution of any disputes.
          </p>
        </section>

        {/* Dispute Resolution */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">10. Dispute Resolution</h2>
          <p className="text-dark-300">
            Before initiating legal action, you agree to contact us and attempt to resolve any dispute informally.
            If informal resolution fails, you agree to attempt resolution through binding arbitration rather than
            court proceedings. However, either party may seek equitable relief (such as injunction) in court to protect
            intellectual property or confidential information.
          </p>
        </section>

        {/* Prohibited Conduct */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">11. Prohibited Conduct</h2>
          <p className="text-dark-300 mb-4">
            You agree not to use the Service for any unlawful or prohibited purpose, including:
          </p>
          <ul className="list-disc list-inside space-y-1 text-dark-300 ml-2">
            <li>Violating any laws or regulations</li>
            <li>Infringing intellectual property rights</li>
            <li>Transmitting malware, viruses, or malicious code</li>
            <li>Attempting to gain unauthorized access</li>
            <li>Harassment, abuse, or threats against any person</li>
            <li>Spam or unsolicited communications</li>
            <li>Attempting to disrupt or interfere with the Service</li>
          </ul>
        </section>

        {/* SLA and Uptime */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">12. Service Availability</h2>
          <p className="text-dark-300">
            While we strive for high availability, we do not guarantee uninterrupted service. We may perform maintenance,
            updates, or upgrades that temporarily affect availability. We shall not be liable for any downtime or service
            interruptions beyond our reasonable control, including natural disasters, infrastructure failures, or DDoS attacks.
          </p>
        </section>

        {/* Changes to Terms */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">13. Changes to These Terms</h2>
          <p className="text-dark-300">
            We may modify these Terms at any time. Changes will be effective immediately upon posting to the Service.
            Your continued use of the Service following modifications constitutes acceptance of the updated Terms.
            We will notify you of material changes via email or prominent notice on the Service.
          </p>
        </section>

        {/* Contact */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">14. Contact Us</h2>
          <p className="text-dark-300 mb-4">
            If you have questions about these Terms of Service, please contact us at:
          </p>
          <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 text-dark-300">
            <p className="font-semibold text-dark-100 mb-2">Knol</p>
            <p className="mb-1">Email: <a href="mailto:hello@aiknol.com" className="text-brand-400 hover:text-brand-300">hello@aiknol.com</a></p>
            <p>GitHub: <a href="https://github.com/aiknol/knol" className="text-brand-400 hover:text-brand-300" target="_blank" rel="noopener noreferrer">github.com/aiknol/knol</a></p>
          </div>
        </section>
      </div>
    </div>
  );
}
